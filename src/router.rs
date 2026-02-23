use jmap_client::Mailbox;

const WELL_KNOWN_ROLES: &[&str] = &["inbox", "drafts", "sent", "junk", "trash", "archive"];

/// Convert a mailbox ID to a URL slug. Well-known roles use the role name;
/// other mailboxes use the raw JMAP ID.
pub fn mailbox_id_to_slug(mailboxes: &[Mailbox], mailbox_id: &str) -> String {
    mailboxes
        .iter()
        .find(|m| m.id == mailbox_id)
        .and_then(|m| {
            m.role.as_deref().and_then(|role| {
                if WELL_KNOWN_ROLES.contains(&role) {
                    Some(role.to_string())
                } else {
                    None
                }
            })
        })
        .unwrap_or_else(|| mailbox_id.to_string())
}

/// Convert a URL slug back to a mailbox ID. Tries role match first, then raw ID.
pub fn slug_to_mailbox_id(mailboxes: &[Mailbox], slug: &str) -> Option<String> {
    if WELL_KNOWN_ROLES.contains(&slug) {
        if let Some(m) = mailboxes.iter().find(|m| m.role.as_deref() == Some(slug)) {
            return Some(m.id.clone());
        }
    }
    if mailboxes.iter().any(|m| m.id == slug) {
        return Some(slug.to_string());
    }
    None
}
