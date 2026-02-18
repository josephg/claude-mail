use crate::components::{
    compose::ComposeView, email_list::EmailList, mailbox_sidebar::MailboxSidebar,
    thread_view::ThreadView,
};
use crate::state::{AppState, MailView};
use leptos::prelude::*;

#[component]
pub fn MailPage() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState to be provided");

    let on_compose = move |_| {
        state.current_view.set(MailView::Compose {
            to: String::new(),
            cc: String::new(),
            bcc: String::new(),
            subject: String::new(),
            body: String::new(),
        });
    };

    view! {
        <div class="mail-layout">
            <div class="mail-toolbar">
                <button class="compose-btn" on:click=on_compose>"Compose"</button>
            </div>
            <div class="mail-content">
                <div class="mail-sidebar">
                    <MailboxSidebar/>
                </div>
                <div class="mail-main">
                    {move || {
                        match state.current_view.get() {
                            MailView::EmailList => view! { <EmailList/> }.into_any(),
                            MailView::ThreadView { thread_id } => view! {
                                <ThreadView thread_id=thread_id/>
                            }.into_any(),
                            MailView::Compose { .. } => view! { <ComposeView/> }.into_any(),
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
