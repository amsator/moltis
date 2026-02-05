//! Access control for WhatsApp messages.

use {
    moltis_channels::gating::{self, DmPolicy, GroupPolicy},
    moltis_common::types::ChatType,
};

use crate::config::WhatsAppAccountConfig;

/// Determine if an inbound message should be processed.
///
/// Returns `Ok(())` if the message is allowed, or `Err(reason)` if it should
/// be silently dropped.
pub fn check_access(
    config: &WhatsAppAccountConfig,
    chat_type: &ChatType,
    peer_id: &str,
    group_id: Option<&str>,
) -> Result<(), AccessDenied> {
    match chat_type {
        ChatType::Dm => check_dm_access(config, peer_id),
        ChatType::Group | ChatType::Channel => check_group_access(config, group_id),
    }
}

fn check_dm_access(config: &WhatsAppAccountConfig, peer_id: &str) -> Result<(), AccessDenied> {
    match config.dm_policy {
        DmPolicy::Disabled => Err(AccessDenied::DmsDisabled),
        DmPolicy::Open => Ok(()),
        DmPolicy::Allowlist => {
            if gating::is_allowed(peer_id, &config.allowlist) {
                Ok(())
            } else {
                Err(AccessDenied::NotOnAllowlist)
            }
        },
    }
}

fn check_group_access(
    config: &WhatsAppAccountConfig,
    group_id: Option<&str>,
) -> Result<(), AccessDenied> {
    match config.group_policy {
        GroupPolicy::Disabled => Err(AccessDenied::GroupsDisabled),
        GroupPolicy::Open => Ok(()),
        GroupPolicy::Allowlist => {
            let gid = group_id.unwrap_or("");
            if gating::is_allowed(gid, &config.group_allowlist) {
                Ok(())
            } else {
                Err(AccessDenied::GroupNotOnAllowlist)
            }
        },
    }
}

/// Reason an inbound message was denied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessDenied {
    DmsDisabled,
    NotOnAllowlist,
    GroupsDisabled,
    GroupNotOnAllowlist,
}

impl std::fmt::Display for AccessDenied {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DmsDisabled => write!(f, "DMs are disabled"),
            Self::NotOnAllowlist => write!(f, "user not on allowlist"),
            Self::GroupsDisabled => write!(f, "groups are disabled"),
            Self::GroupNotOnAllowlist => write!(f, "group not on allowlist"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> WhatsAppAccountConfig {
        WhatsAppAccountConfig::default()
    }

    #[test]
    fn open_dm_allows_all() {
        let c = cfg();
        assert!(check_access(&c, &ChatType::Dm, "15551234567", None).is_ok());
    }

    #[test]
    fn disabled_dm_rejects() {
        let mut c = cfg();
        c.dm_policy = DmPolicy::Disabled;
        assert_eq!(
            check_access(&c, &ChatType::Dm, "15551234567", None),
            Err(AccessDenied::DmsDisabled)
        );
    }

    #[test]
    fn allowlist_dm() {
        let mut c = cfg();
        c.dm_policy = DmPolicy::Allowlist;
        c.allowlist = vec!["15551234567".into()];
        assert!(check_access(&c, &ChatType::Dm, "15551234567", None).is_ok());
        assert_eq!(
            check_access(&c, &ChatType::Dm, "15559999999", None),
            Err(AccessDenied::NotOnAllowlist)
        );
    }

    #[test]
    fn allowlist_dm_with_wildcard() {
        let mut c = cfg();
        c.dm_policy = DmPolicy::Allowlist;
        c.allowlist = vec!["1555*".into()];
        assert!(check_access(&c, &ChatType::Dm, "15551234567", None).is_ok());
        assert_eq!(
            check_access(&c, &ChatType::Dm, "14151234567", None),
            Err(AccessDenied::NotOnAllowlist)
        );
    }

    #[test]
    fn group_disabled() {
        let mut c = cfg();
        c.group_policy = GroupPolicy::Disabled;
        assert_eq!(
            check_access(&c, &ChatType::Group, "user", Some("grp1")),
            Err(AccessDenied::GroupsDisabled)
        );
    }

    #[test]
    fn group_allowlist() {
        let mut c = cfg();
        c.group_policy = GroupPolicy::Allowlist;
        c.group_allowlist = vec!["grp1".into()];
        assert!(check_access(&c, &ChatType::Group, "user", Some("grp1")).is_ok());
        assert_eq!(
            check_access(&c, &ChatType::Group, "user", Some("grp2")),
            Err(AccessDenied::GroupNotOnAllowlist)
        );
    }
}
