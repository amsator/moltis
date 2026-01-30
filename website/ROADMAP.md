# Moltis Roadmap

Moltis is a Rust rewrite of moltbot — a personal AI gateway that connects LLMs to your messaging channels, tools, and devices. This roadmap tracks feature parity with the original and new capabilities.

## Status Legend

- **Done** — Implemented and working
- **Partial** — Infrastructure exists, needs completion
- **Planned** — Not yet started

---

## Core Gateway

| Feature | Status | Notes |
|---|---|---|
| WebSocket protocol v3 | Done | JSON-RPC frames, handshake, heartbeat |
| HTTP health endpoint | Done | |
| Embedded web UI | Done | Tailwind CSS, dark/light theme, chat + method explorer |
| CORS support | Done | |
| Connection deduplication | Done | 5-minute TTL |
| Max payload 512 KB | Done | |

## LLM Providers

| Feature | Status | Notes |
|---|---|---|
| Anthropic Claude | Done | claude-sonnet-4-20250514 |
| OpenAI GPT-4o | Done | Built-in + async-openai + genai |
| OpenAI Codex (OAuth) | Done | PKCE flow, device tokens |
| Google Gemini | Done | Via genai |
| Groq Llama | Done | Via genai |
| xAI Grok | Done | Via genai |
| DeepSeek | Done | Via genai |
| Dynamic model discovery | Partial | Config-based, env vars |
| Provider plugin API | Partial | Trait defined, loading incomplete |

## Agent System

| Feature | Status | Notes |
|---|---|---|
| Agent loop with tool calling | Done | Max 25 iterations |
| Streaming completions | Done | Delta events via WebSocket |
| System prompt building | Done | |
| Auth profiles per agent | Done | |
| Multi-agent routing | Done | Binding cascade: peer > guild > team > account > channel > default |
| Agent skills injection | Partial | Infrastructure exists |
| Agent wait (async invoke) | Done | |

## Tool System

| Feature | Status | Notes |
|---|---|---|
| Tool registry (trait-based) | Done | Pluggable architecture |
| Exec tool (shell commands) | Done | Timeout, output limits, allowlist |
| Security layers | Done | Multi-layer allow/deny policies, approval modes |
| Browser automation | Partial | CDP infrastructure, needs pool manager |
| Canvas (agent-controlled UI) | Partial | HTTP + WebSocket server defined |
| Memory / RAG | Partial | SQLite + sqlite-vec, needs full backend |
| Web fetch | Planned | |
| Web search | Planned | |
| Image generation | Planned | |
| Message tool | Planned | Send messages within sessions |
| Node invocation tool | Planned | Remote execution on devices |
| Cron tool | Partial | Scheduling infrastructure exists |
| Session tools | Partial | CRUD defined |

## Sandbox

| Feature | Status | Notes |
|---|---|---|
| Docker-based isolation | Partial | Schema and config ready, runtime pending |
| Session/agent/shared scopes | Partial | |
| Network isolation | Partial | |
| Workspace mount control | Partial | ReadOnly default |

## Channels

| Feature | Status | Notes |
|---|---|---|
| Channel plugin trait | Done | Pluggable architecture |
| Telegram | Planned | |
| Discord | Planned | |
| Slack | Planned | |
| WhatsApp | Planned | |
| SMS | Planned | |
| Email | Planned | |
| Signal | Planned | |
| Channel health probing | Partial | Trait defined |
| Media attachments | Partial | Pipeline exists |

## Sessions

| Feature | Status | Notes |
|---|---|---|
| JSONL storage | Done | File locking, concurrent access |
| Session key routing | Done | Per-peer, guild, team, account, channel |
| Compaction | Partial | |
| History retrieval | Done | |
| Session reset/delete | Done | |

## Auto-Reply Pipeline

| Feature | Status | Notes |
|---|---|---|
| Message normalization | Partial | |
| Route resolution | Done | Binding cascade |
| Session loading | Done | |
| Media understanding | Partial | |
| Directive parsing (#think, #exec, #reset) | Planned | Infrastructure in place |
| Response chunking | Partial | |
| Channel delivery | Partial | |

## Media Pipeline

| Feature | Status | Notes |
|---|---|---|
| Download and storage | Partial | |
| MIME detection | Partial | Needs sniffing + header fallback |
| Image resize | Partial | |
| Audio transcription | Planned | |
| TTL-based cleanup | Partial | |
| HTTP serving | Partial | |

## Authentication & Security

| Feature | Status | Notes |
|---|---|---|
| Token auth | Done | MOLTIS_TOKEN env var |
| Password auth | Done | |
| Device pairing (PKI) | Done | State machine, scoped tokens |
| OAuth PKCE flow | Done | OpenAI Codex provider |
| Role-based access | Done | Operator, node roles |
| Scoped permissions | Done | Admin, read, write, approvals, pairing |
| Tool approval workflow | Done | |

## Configuration

| Feature | Status | Notes |
|---|---|---|
| Multi-format (TOML/YAML/JSON) | Done | |
| Env var substitution | Done | ${ENV_VAR} |
| Provider enable/disable | Done | |
| Config get/set/edit via gateway | Done | |
| Legacy migration | Partial | |

## Plugin System

| Feature | Status | Notes |
|---|---|---|
| Plugin trait API | Done | Tools, channels, hooks, providers, commands, routes |
| Bundled plugin discovery | Partial | |
| Global/workspace plugin paths | Partial | |
| Plugin loading from disk | Planned | |

## CLI

| Feature | Status | Notes |
|---|---|---|
| gateway command | Done | --bind, --port |
| agent command | Done | --message, --think |
| channels command | Done | status, login, logout |
| send command | Done | |
| sessions command | Done | list, clear, history |
| config command | Done | get, set, edit |
| models command | Done | |
| onboard wizard | Partial | Flow defined |
| doctor command | Partial | |
| auth command | Done | OAuth management |

## Scheduling (Cron)

| Feature | Status | Notes |
|---|---|---|
| Cron expression parsing | Done | |
| Persistent job storage | Done | |
| Isolated agent execution | Partial | |
| Channel delivery | Planned | |
| Execution history | Partial | |

## Node System

| Feature | Status | Notes |
|---|---|---|
| Node registration & pairing | Done | |
| Remote invocation | Partial | |
| Node capabilities tracking | Partial | |
| Canvas host for mobile | Partial | |

## Observability

| Feature | Status | Notes |
|---|---|---|
| Structured logging (tracing) | Done | JSON + human-readable |
| OpenTelemetry support | Partial | OTLP dependency present |
| Usage/cost tracking | Planned | |

## Text-to-Speech

| Feature | Status | Notes |
|---|---|---|
| TTS provider framework | Planned | Protocol methods defined |
| Voice wake word | Planned | Protocol methods defined |

---

## What's Next

Near-term priorities:
1. Complete channel implementations (Telegram and Discord first)
2. Finish browser automation with CDP pool manager
3. Build out web fetch and web search tools
4. Complete memory/RAG backend
5. Ship plugin loading from disk
6. Add image generation tool
7. Directive parsing (#think, #exec, #reset)
8. TTS and voice wake word support

Long-term goals:
- Full feature parity with moltbot
- Native mobile node SDK
- Plugin marketplace
- Multi-user support with teams
- End-to-end encryption for sessions
