use crate::pages::{login::LoginPage, mail::MailPage};
use crate::state::{load_saved_credentials, AppState};
use jmap_client::JmapClient;
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn App() -> impl IntoView {
    let state = AppState::new();
    provide_context(state);

    // Try auto-login from saved credentials
    if let Some((server, username, password)) = load_saved_credentials() {
        spawn_local(async move {
            if let Ok(client) = JmapClient::connect(&server, &username, &password).await {
                let mailboxes = client.get_mailboxes().await.ok().unwrap_or_default();
                let identities = client.get_identities().await.ok().unwrap_or_default();
                let inbox_id = mailboxes
                    .iter()
                    .find(|m| m.role.as_deref() == Some("inbox"))
                    .map(|m| m.id.clone());

                state.mailboxes.set(mailboxes);
                state.identities.set(identities);
                state.client.set(Some(client));

                if let Some(id) = inbox_id {
                    state.selected_mailbox.set(Some(id));
                }
            }
        });
    }

    view! {
        {move || {
            if state.client.get().is_some() {
                view! { <MailPage/> }.into_any()
            } else {
                view! { <LoginPage/> }.into_any()
            }
        }}
    }
}
