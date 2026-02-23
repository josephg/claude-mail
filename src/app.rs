use crate::components::{compose::ComposeView, email_list::EmailList, thread_view::ThreadView};
use crate::pages::{login::LoginPage, mail::MailLayout};
use crate::state::{load_saved_credentials, AppState};
use jmap_client::JmapClient;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::{ParentRoute, Redirect, Route, Router, Routes};
use leptos_router::path;

#[component]
pub fn App() -> impl IntoView {
    let state = AppState::new();
    provide_context(state);

    // Try auto-login from saved credentials
    if let Some((server, username, password)) = load_saved_credentials() {
        spawn_local(async move {
            if let Ok(client) = JmapClient::connect(&server, &username, &password).await {
                let (mailboxes, mailbox_state) =
                    client.get_mailboxes().await.ok().unwrap_or_default();
                let identities = client.get_identities().await.ok().unwrap_or_default();

                state.mailboxes.set(mailboxes);
                state.mailbox_state.set(Some(mailbox_state));
                state.identities.set(identities);
                state.client.set(Some(client));

                crate::eventsource::start_event_source(state);
            }
            state.auto_login_done.set(true);
        });
    } else {
        state.auto_login_done.set(true);
    }

    view! {
        <Router>
            <Routes fallback=|| view! { <Redirect path="/login"/> }>
                <Route path=path!("/") view=|| view! { <Redirect path="/mail/inbox"/> }/>
                <Route path=path!("/login") view=LoginPage/>
                <ParentRoute path=path!("/mail/:mailbox") view=MailLayout>
                    <Route path=path!("") view=EmailList/>
                    <Route path=path!("/compose") view=ComposeView/>
                    <Route path=path!("/:thread_id") view=ThreadView/>
                </ParentRoute>
            </Routes>
        </Router>
    }
}
