use crate::state::AppState;
use jmap_client::StateChange;
use leptos::prelude::*;
use leptos::task::spawn_local;

pub fn handle_state_change(state: AppState, change: StateChange) {
    let Some(client) = state.client.get_untracked() else {
        return;
    };
    let account_id = client.account_id().to_string();

    let Some(type_changes) = change.changed.get(&account_id) else {
        return;
    };

    let mailbox_changed = type_changes.contains_key("Mailbox");
    let email_changed = type_changes.contains_key("Email");

    if mailbox_changed {
        spawn_local(async move {
            let Some(client) = state.client.get_untracked() else {
                return;
            };
            if let Ok((mailboxes, mailbox_state)) = client.get_mailboxes().await {
                state.mailboxes.set(mailboxes);
                state.mailbox_state.set(Some(mailbox_state));
            }
        });
    }

    if email_changed {
        let new_email_state = type_changes.get("Email").cloned();
        // Bump the refresh trigger so the email list LocalResource re-runs
        state
            .email_refresh_trigger
            .update(|v| *v = v.wrapping_add(1));
        // Update tracked email state
        if let Some(es) = new_email_state {
            state.email_state.set(Some(es));
        }
    }
}
