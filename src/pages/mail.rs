use crate::components::mailbox_sidebar::MailboxSidebar;
use crate::router::{mailbox_id_to_slug, slug_to_mailbox_id};
use crate::state::AppState;
use leptos::prelude::*;
use leptos_router::components::{Outlet, Redirect};
use leptos_router::hooks::{use_navigate, use_params_map};

#[component]
pub fn MailLayout() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState to be provided");
    let params = use_params_map();
    let navigate = use_navigate();

    // Sync mailbox from URL param → signal
    Effect::new(move || {
        let p = params.read();
        let slug = p.get("mailbox").unwrap_or_default();
        let mailboxes = state.mailboxes.get();
        if let Some(id) = slug_to_mailbox_id(&mailboxes, &slug) {
            if state.selected_mailbox.get_untracked().as_deref() != Some(id.as_str()) {
                state.selected_mailbox.set(Some(id));
            }
        }
    });

    view! {
        {move || {
            let client = state.client.get();
            let auto_done = state.auto_login_done.get();

            if client.is_none() && !auto_done {
                return view! { <div class="loading">"Connecting..."</div> }.into_any();
            }
            if client.is_none() {
                return view! { <Redirect path="/login"/> }.into_any();
            }

            let nav = navigate.clone();
            let on_compose = move |_| {
                let mailboxes = state.mailboxes.get();
                let slug = state
                    .selected_mailbox
                    .get()
                    .as_deref()
                    .map(|id| mailbox_id_to_slug(&mailboxes, id))
                    .unwrap_or_else(|| "inbox".to_string());
                nav(&format!("/mail/{slug}/compose"), Default::default());
            };

            let nav = navigate.clone();
            let on_logout = move |_| {
                state.logout();
                nav("/login", Default::default());
            };

            view! {
                <div class="mail-layout">
                    <div class="mail-toolbar">
                        <button class="compose-btn" on:click=on_compose>"Compose"</button>
                        <div class="toolbar-spacer"></div>
                        <button class="logout-btn" on:click=on_logout>"Logout"</button>
                    </div>
                    <div class="mail-content">
                        <div class="mail-sidebar">
                            <MailboxSidebar/>
                        </div>
                        <div class="mail-main">
                            <Outlet/>
                        </div>
                    </div>
                </div>
            }.into_any()
        }}
    }
}
