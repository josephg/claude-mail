use crate::components::compose::ComposeInline;
use crate::state::{AppState, MailView};
use jmap_client::Email;
use leptos::prelude::*;

#[component]
pub fn ThreadView(thread_id: String) -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState to be provided");

    let thread_id_for_fetch = thread_id.clone();
    let emails = LocalResource::new(move || {
        let client = state.client.get();
        let tid = thread_id_for_fetch.clone();
        async move {
            let Some(client) = client else {
                return Vec::<Email>::new();
            };
            let thread = match client.get_thread(&tid).await {
                Ok(t) => t,
                Err(_) => return vec![],
            };
            client
                .get_email_bodies(&thread.email_ids)
                .await
                .ok()
                .unwrap_or_default()
        }
    });

    let on_back = move |_| {
        state.current_view.set(MailView::EmailList);
        state.reply_to_email.set(None);
    };

    view! {
        <div class="thread-view">
            <div class="thread-toolbar">
                <button on:click=on_back>"Back"</button>
            </div>
            {move || {
                match emails.get() {
                    None => view! { <div class="loading">"Loading thread..."</div> }.into_any(),
                    Some(email_list) => {
                        email_list.iter().map(|email| {
                            let email_id = email.id.clone().unwrap_or_default();
                            view! {
                                <EmailCard email=email.clone()/>
                                {move || {
                                    let reply_id = state.reply_to_email.get();
                                    if reply_id.as_deref() == Some(&email_id) {
                                        Some(view! { <ComposeInline email_id=email_id.clone()/> })
                                    } else {
                                        None
                                    }
                                }}
                            }
                        }).collect_view().into_any()
                    }
                }
            }}
        </div>
    }
}

#[component]
fn EmailCard(email: Email) -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState to be provided");
    let email_id = email.id.clone().unwrap_or_default();
    let email_id_reply = email_id.clone();
    let email_id_reply_all = email_id.clone();

    let from = email
        .from
        .as_ref()
        .map(|addrs| {
            addrs
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_else(|| "(unknown)".to_string());

    let to = email
        .to
        .as_ref()
        .map(|addrs| {
            addrs
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();

    let date = email.received_at.clone().unwrap_or_default();
    let subject = email.subject.clone().unwrap_or_default();

    let body_text = email
        .text_body
        .as_ref()
        .and_then(|parts| parts.first())
        .and_then(|part| part.part_id.as_ref())
        .and_then(|part_id| {
            email
                .body_values
                .as_ref()
                .and_then(|bv| bv.get(part_id))
                .map(|v| v.value.clone())
        })
        .unwrap_or_else(|| "(no text content)".to_string());

    let on_reply = move |_| {
        state.reply_to_email.set(Some(email_id_reply.clone()));
        state.reply_all.set(false);
    };

    let on_reply_all = move |_| {
        state.reply_to_email.set(Some(email_id_reply_all.clone()));
        state.reply_all.set(true);
    };

    view! {
        <div class="email-card">
            <div class="email-card-header">
                <div class="email-card-from"><strong>"From: "</strong>{from}</div>
                <div class="email-card-to"><strong>"To: "</strong>{to}</div>
                <div class="email-card-date"><strong>"Date: "</strong>{date}</div>
                <div class="email-card-subject"><strong>"Subject: "</strong>{subject}</div>
            </div>
            <div class="email-card-body">
                <pre>{body_text}</pre>
            </div>
            <div class="email-card-actions">
                <button on:click=on_reply>"Reply"</button>
                <button on:click=on_reply_all>"Reply All"</button>
            </div>
        </div>
    }
}
