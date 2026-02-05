use std::{net::SocketAddr, sync::Arc};

use {
    anyhow::{Result, bail},
    tokio::{
        io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
        net::{TcpListener, TcpStream},
    },
    tracing::{debug, info, instrument, warn},
};

#[cfg(feature = "metrics")]
use moltis_metrics::{counter, gauge, histogram};

use crate::domain_approval::{DomainApprovalManager, DomainDecision, DomainFilter, FilterAction};

/// The default port the proxy listens on inside the trusted network.
pub const DEFAULT_PROXY_PORT: u16 = 18791;

/// HTTP CONNECT proxy server that filters outbound connections by domain.
///
/// The proxy handles:
/// - `CONNECT host:port` requests (used by HTTPS clients via `HTTPS_PROXY`)
/// - Plain HTTP requests forwarded via `HTTP_PROXY`
///
/// For each connection, the domain is checked against the `DomainApprovalManager`.
/// If the domain needs approval, the proxy holds the connection until a decision
/// is made (or timeout).
pub struct NetworkProxyServer {
    listener_addr: SocketAddr,
    filter: Arc<DomainApprovalManager>,
}

impl NetworkProxyServer {
    pub fn new(listener_addr: SocketAddr, filter: Arc<DomainApprovalManager>) -> Self {
        Self {
            listener_addr,
            filter,
        }
    }

    /// Start the proxy server. This runs until the `shutdown` future completes.
    pub async fn run(&self, shutdown: tokio::sync::watch::Receiver<bool>) -> Result<()> {
        let listener = TcpListener::bind(self.listener_addr).await?;
        info!(addr = %self.listener_addr, "network proxy listening");

        loop {
            tokio::select! {
                accept = listener.accept() => {
                    match accept {
                        Ok((stream, peer)) => {
                            let filter = Arc::clone(&self.filter);
                            tokio::spawn(async move {
                                if let Err(e) = handle_client(stream, peer, filter).await {
                                    debug!(peer = %peer, error = %e, "proxy client error");
                                }
                            });
                        },
                        Err(e) => {
                            warn!(error = %e, "proxy accept error");
                        },
                    }
                },
                _ = shutdown_signal(&shutdown) => {
                    info!("network proxy shutting down");
                    break;
                },
            }
        }
        Ok(())
    }

    /// The address the proxy is listening on.
    pub fn addr(&self) -> SocketAddr {
        self.listener_addr
    }
}

async fn shutdown_signal(rx: &tokio::sync::watch::Receiver<bool>) {
    let mut rx = rx.clone();
    while !*rx.borrow_and_update() {
        if rx.changed().await.is_err() {
            return;
        }
    }
}

/// Handle a single client connection.
///
/// Reads the first line to determine if it's a CONNECT request or a plain HTTP request.
#[instrument(skip(stream, filter), fields(peer = %peer))]
async fn handle_client(
    stream: TcpStream,
    peer: SocketAddr,
    filter: Arc<DomainApprovalManager>,
) -> Result<()> {
    #[cfg(feature = "metrics")]
    {
        counter!("proxy_connections_total").increment(1);
        gauge!("proxy_connections_active").increment(1.0);
    }

    let result = handle_client_inner(stream, peer, filter).await;

    #[cfg(feature = "metrics")]
    gauge!("proxy_connections_active").decrement(1.0);

    result
}

async fn handle_client_inner(
    stream: TcpStream,
    peer: SocketAddr,
    filter: Arc<DomainApprovalManager>,
) -> Result<()> {
    let mut reader = BufReader::new(stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line).await?;
    let request_line = request_line.trim_end();

    if request_line.is_empty() {
        bail!("empty request");
    }

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        bail!("malformed request line: {request_line}");
    }

    let method = parts[0];
    let target = parts[1];

    if method.eq_ignore_ascii_case("CONNECT") {
        handle_connect(reader, peer, target, filter).await
    } else {
        handle_http_forward(reader, peer, method, target, filter).await
    }
}

/// Handle an HTTP CONNECT tunnel request.
#[instrument(skip(reader, filter), fields(peer = %peer, target = %target))]
async fn handle_connect(
    mut reader: BufReader<TcpStream>,
    peer: SocketAddr,
    target: &str,
    filter: Arc<DomainApprovalManager>,
) -> Result<()> {
    #[cfg(feature = "metrics")]
    let start = std::time::Instant::now();

    // Parse host:port from CONNECT target.
    let (domain, port) = parse_host_port(target)?;

    // Consume remaining request headers.
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).await?;
        if line.trim().is_empty() {
            break;
        }
    }

    // Use peer address as session identifier for now.
    let session = peer.to_string();
    let action = filter.check(&session, &domain).await;

    match action {
        FilterAction::Allow => {
            #[cfg(feature = "metrics")]
            counter!("proxy_requests_total", "method" => "CONNECT", "result" => "allowed")
                .increment(1);
        },
        FilterAction::Deny => {
            #[cfg(feature = "metrics")]
            counter!("proxy_requests_total", "method" => "CONNECT", "result" => "denied")
                .increment(1);
            let resp = "HTTP/1.1 403 Forbidden\r\n\r\n";
            reader.get_mut().write_all(resp.as_bytes()).await?;
            return Ok(());
        },
        FilterAction::NeedsApproval => {
            let (id, rx) = filter.create_request(&session, &domain).await;
            debug!(id = %id, domain = %domain, "waiting for domain approval");
            let decision = filter.wait_for_decision(rx).await;
            match decision {
                DomainDecision::Approved => {
                    #[cfg(feature = "metrics")]
                    counter!("proxy_requests_total", "method" => "CONNECT", "result" => "approved")
                        .increment(1);
                },
                DomainDecision::Denied | DomainDecision::Timeout => {
                    #[cfg(feature = "metrics")]
                    counter!("proxy_requests_total", "method" => "CONNECT", "result" => "denied")
                        .increment(1);
                    let resp = "HTTP/1.1 403 Forbidden\r\n\r\n";
                    reader.get_mut().write_all(resp.as_bytes()).await?;
                    return Ok(());
                },
            }
        },
    }

    // Connect to upstream.
    let upstream_addr = format!("{domain}:{port}");
    let upstream = match TcpStream::connect(&upstream_addr).await {
        Ok(s) => s,
        Err(e) => {
            #[cfg(feature = "metrics")]
            counter!("proxy_upstream_errors_total", "error" => "connect_failed").increment(1);
            let resp = format!("HTTP/1.1 502 Bad Gateway\r\n\r\n{e}");
            reader.get_mut().write_all(resp.as_bytes()).await?;
            return Ok(());
        },
    };

    // Send 200 Connection Established.
    let resp = "HTTP/1.1 200 Connection Established\r\n\r\n";
    reader.get_mut().write_all(resp.as_bytes()).await?;

    // Bidirectional copy.
    let mut client_stream = reader.into_inner();
    let (mut client_read, mut client_write) = client_stream.split();
    let (mut upstream_read, mut upstream_write) = upstream.into_split();

    let c2u = tokio::io::copy(&mut client_read, &mut upstream_write);
    let u2c = tokio::io::copy(&mut upstream_read, &mut client_write);

    let (c2u_result, u2c_result) = tokio::join!(c2u, u2c);

    #[cfg(feature = "metrics")]
    {
        if let Ok(bytes) = c2u_result {
            counter!("proxy_bytes_transferred_total", "direction" => "client_to_upstream")
                .increment(bytes);
        }
        if let Ok(bytes) = u2c_result {
            counter!("proxy_bytes_transferred_total", "direction" => "upstream_to_client")
                .increment(bytes);
        }
        histogram!("proxy_tunnel_duration_seconds").record(start.elapsed().as_secs_f64());
    }

    #[cfg(not(feature = "metrics"))]
    {
        if let Err(e) = c2u_result {
            debug!(error = %e, "client->upstream copy ended");
        }
        if let Err(e) = u2c_result {
            debug!(error = %e, "upstream->client copy ended");
        }
    }

    Ok(())
}

/// Handle a plain HTTP forward request (non-CONNECT).
///
/// Used when `HTTP_PROXY` is set and the client sends a full URL request.
#[instrument(skip(reader, filter), fields(peer = %peer, method = %method, target = %target))]
async fn handle_http_forward(
    mut reader: BufReader<TcpStream>,
    peer: SocketAddr,
    method: &str,
    target: &str,
    filter: Arc<DomainApprovalManager>,
) -> Result<()> {
    #[cfg(feature = "metrics")]
    let start = std::time::Instant::now();

    // Extract host from the URL.
    let domain = extract_host_from_url(target)?;
    let port = extract_port_from_url(target);

    let session = peer.to_string();
    let action = filter.check(&session, &domain).await;

    match action {
        FilterAction::Allow => {
            #[cfg(feature = "metrics")]
            counter!("proxy_requests_total", "method" => "HTTP", "result" => "allowed")
                .increment(1);
        },
        FilterAction::Deny => {
            #[cfg(feature = "metrics")]
            counter!("proxy_requests_total", "method" => "HTTP", "result" => "denied").increment(1);
            let resp = "HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\n\r\n";
            reader.get_mut().write_all(resp.as_bytes()).await?;
            return Ok(());
        },
        FilterAction::NeedsApproval => {
            let (id, rx) = filter.create_request(&session, &domain).await;
            debug!(id = %id, domain = %domain, "waiting for domain approval (HTTP)");
            let decision = filter.wait_for_decision(rx).await;
            match decision {
                DomainDecision::Approved => {
                    #[cfg(feature = "metrics")]
                    counter!("proxy_requests_total", "method" => "HTTP", "result" => "approved")
                        .increment(1);
                },
                DomainDecision::Denied | DomainDecision::Timeout => {
                    #[cfg(feature = "metrics")]
                    counter!("proxy_requests_total", "method" => "HTTP", "result" => "denied")
                        .increment(1);
                    let resp = "HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\n\r\n";
                    reader.get_mut().write_all(resp.as_bytes()).await?;
                    return Ok(());
                },
            }
        },
    }

    // Read remaining headers.
    let mut headers = String::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).await?;
        if line.trim().is_empty() {
            break;
        }
        headers.push_str(&line);
    }

    // Connect to upstream and forward the request.
    let upstream_addr = format!("{domain}:{port}");
    let mut upstream = match TcpStream::connect(&upstream_addr).await {
        Ok(s) => s,
        Err(e) => {
            #[cfg(feature = "metrics")]
            counter!("proxy_upstream_errors_total", "error" => "connect_failed").increment(1);
            let resp = format!("HTTP/1.1 502 Bad Gateway\r\n\r\n{e}");
            reader.get_mut().write_all(resp.as_bytes()).await?;
            return Ok(());
        },
    };

    // Convert absolute URL to relative path for upstream.
    let path = url_to_path(target);
    let request_line = format!("{method} {path} HTTP/1.1\r\n");
    upstream.write_all(request_line.as_bytes()).await?;
    upstream.write_all(headers.as_bytes()).await?;
    upstream.write_all(b"\r\n").await?;

    // Bidirectional copy for the rest.
    let mut client_stream = reader.into_inner();
    let (mut client_read, mut client_write) = client_stream.split();
    let (mut upstream_read, mut upstream_write) = upstream.into_split();

    let c2u = tokio::io::copy(&mut client_read, &mut upstream_write);
    let u2c = tokio::io::copy(&mut upstream_read, &mut client_write);

    let (c2u_result, u2c_result) = tokio::join!(c2u, u2c);

    #[cfg(feature = "metrics")]
    {
        if let Ok(bytes) = c2u_result {
            counter!("proxy_bytes_transferred_total", "direction" => "client_to_upstream")
                .increment(bytes);
        }
        if let Ok(bytes) = u2c_result {
            counter!("proxy_bytes_transferred_total", "direction" => "upstream_to_client")
                .increment(bytes);
        }
        histogram!("proxy_request_duration_seconds").record(start.elapsed().as_secs_f64());
    }

    #[cfg(not(feature = "metrics"))]
    {
        if let Err(e) = c2u_result {
            debug!(error = %e, "client->upstream copy ended");
        }
        if let Err(e) = u2c_result {
            debug!(error = %e, "upstream->client copy ended");
        }
    }

    Ok(())
}

/// Parse `host:port` from a CONNECT target. Defaults port to 443 if not specified.
fn parse_host_port(target: &str) -> Result<(String, u16)> {
    if let Some((host, port_str)) = target.rsplit_once(':') {
        let port: u16 = port_str.parse().unwrap_or(443);
        Ok((host.to_string(), port))
    } else {
        Ok((target.to_string(), 443))
    }
}

/// Extract the hostname from an absolute HTTP URL.
fn extract_host_from_url(url: &str) -> Result<String> {
    // Strip scheme.
    let after_scheme = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .unwrap_or(url);

    // Take up to the first '/' or end.
    let host_port = after_scheme.split('/').next().unwrap_or(after_scheme);

    // Strip port if present.
    if let Some((host, _)) = host_port.rsplit_once(':') {
        Ok(host.to_string())
    } else {
        Ok(host_port.to_string())
    }
}

/// Extract the port from an absolute HTTP URL. Defaults to 80 for http, 443 for https.
fn extract_port_from_url(url: &str) -> u16 {
    let is_https = url.starts_with("https://");
    let after_scheme = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .unwrap_or(url);

    let host_port = after_scheme.split('/').next().unwrap_or(after_scheme);
    if let Some((_, port_str)) = host_port.rsplit_once(':') {
        port_str.parse().unwrap_or(if is_https {
            443
        } else {
            80
        })
    } else if is_https {
        443
    } else {
        80
    }
}

/// Convert an absolute URL to a relative path.
fn url_to_path(url: &str) -> String {
    let after_scheme = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .unwrap_or(url);

    if let Some(slash_pos) = after_scheme.find('/') {
        after_scheme[slash_pos..].to_string()
    } else {
        "/".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_host_port() {
        let (host, port) = parse_host_port("github.com:443").unwrap();
        assert_eq!(host, "github.com");
        assert_eq!(port, 443);

        let (host, port) = parse_host_port("example.com").unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(port, 443);

        let (host, port) = parse_host_port("api.example.com:8080").unwrap();
        assert_eq!(host, "api.example.com");
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_extract_host_from_url() {
        assert_eq!(
            extract_host_from_url("http://example.com/path").unwrap(),
            "example.com"
        );
        assert_eq!(
            extract_host_from_url("https://api.github.com:443/v1").unwrap(),
            "api.github.com"
        );
        assert_eq!(
            extract_host_from_url("http://localhost:8080/").unwrap(),
            "localhost"
        );
    }

    #[test]
    fn test_extract_port_from_url() {
        assert_eq!(extract_port_from_url("http://example.com/path"), 80);
        assert_eq!(extract_port_from_url("https://example.com/path"), 443);
        assert_eq!(extract_port_from_url("http://example.com:8080/path"), 8080);
    }

    #[test]
    fn test_url_to_path() {
        assert_eq!(
            url_to_path("http://example.com/path/to/resource"),
            "/path/to/resource"
        );
        assert_eq!(url_to_path("http://example.com"), "/");
        assert_eq!(url_to_path("https://api.github.com/v1/repos"), "/v1/repos");
    }
}
