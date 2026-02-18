use crate::state::{save_credentials, AppState};
use jmap_client::JmapClient;
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn LoginPage() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState to be provided");
    let server_url = RwSignal::new(String::new());
    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let error_msg = RwSignal::new(Option::<String>::None);
    let loading = RwSignal::new(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let server = server_url.get();
        let user = username.get();
        let pass = password.get();

        loading.set(true);
        error_msg.set(None);

        spawn_local(async move {
            match JmapClient::connect(&server, &user, &pass).await {
                Ok(client) => {
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

                    save_credentials(&server, &user, &pass);
                }
                Err(e) => {
                    error_msg.set(Some(format!("{e}")));
                    loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="login-container">
            <h1>"JMAP Webmail"</h1>
            <form class="login-form" on:submit=on_submit>
                <div class="form-field">
                    <label for="server">"Server URL"</label>
                    <input
                        id="server"
                        type="text"
                        placeholder="https://jmap.example.com"
                        bind:value=server_url
                    />
                </div>
                <div class="form-field">
                    <label for="username">"Username"</label>
                    <input
                        id="username"
                        type="text"
                        placeholder="user@example.com"
                        bind:value=username
                    />
                </div>
                <div class="form-field">
                    <label for="password">"Password"</label>
                    <input
                        id="password"
                        type="password"
                        bind:value=password
                    />
                </div>
                <button type="submit" disabled=move || loading.get()>
                    {move || if loading.get() { "Connecting..." } else { "Login" }}
                </button>
                {move || error_msg.get().map(|msg| view! {
                    <div class="error-message">{msg}</div>
                })}
            </form>
        </div>
    }
}
