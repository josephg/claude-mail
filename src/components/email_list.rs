use crate::state::{AppState, MailView};
use jmap_client::Email;
use leptos::prelude::*;
use std::collections::HashMap;

const LIST_PROPERTIES: &[&str] = &[
    "id",
    "threadId",
    "from",
    "subject",
    "receivedAt",
    "preview",
    "keywords",
    "hasAttachment",
];

#[component]
pub fn EmailList() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState to be provided");

    let emails = LocalResource::new(move || {
        let mailbox_id = state.selected_mailbox.get();
        let client = state.client.get();
        async move {
            let (Some(client), Some(mailbox_id)) = (client, mailbox_id) else {
                return Vec::<Email>::new();
            };
            let (ids, _total) = client
                .query_emails(&mailbox_id, 0, 50)
                .await
                .ok()
                .unwrap_or_default();
            if ids.is_empty() {
                return vec![];
            }
            client
                .get_emails(&ids, LIST_PROPERTIES)
                .await
                .ok()
                .unwrap_or_default()
        }
    });

    view! {
        <div class="email-list">
            {move || {
                match emails.get() {
                    None => view! { <div class="loading">"Loading..."</div> }.into_any(),
                    Some(email_list) => {
                        if email_list.is_empty() {
                            view! { <div class="empty">"No emails in this mailbox"</div> }.into_any()
                        } else {
                            email_list.iter().map(|email| {
                                let thread_id = email.thread_id.clone().unwrap_or_default();
                                let subject = email.subject.clone().unwrap_or_else(|| "(no subject)".to_string());
                                let preview = email.preview.clone().unwrap_or_default();
                                let from = email.from.as_ref()
                                    .and_then(|addrs| addrs.first())
                                    .map(|a| a.name.as_deref().unwrap_or(&a.email).to_string())
                                    .unwrap_or_else(|| "(unknown)".to_string());
                                let date = email.received_at.clone().unwrap_or_default();
                                let keywords: &HashMap<String, bool> = match &email.keywords {
                                    Some(kw) => kw,
                                    None => &HashMap::new(),
                                };
                                let is_unread = !keywords.contains_key("$seen");
                                let has_attachment = email.has_attachment.unwrap_or(false);

                                let on_click = {
                                    let thread_id = thread_id.clone();
                                    move |_| {
                                        state.current_view.set(MailView::ThreadView {
                                            thread_id: thread_id.clone(),
                                        });
                                    }
                                };

                                view! {
                                    <div
                                        class="email-row"
                                        class:unread=is_unread
                                        on:click=on_click
                                    >
                                        <div class="email-from">{from}</div>
                                        <div class="email-subject-preview">
                                            <span class="email-subject">{subject}</span>
                                            {if has_attachment {
                                                Some(view! { <span class="attachment-icon">" [att]"</span> })
                                            } else {
                                                None
                                            }}
                                            <span class="email-preview">" - " {preview}</span>
                                        </div>
                                        <div class="email-date">{format_date(&date)}</div>
                                    </div>
                                }
                            }).collect_view().into_any()
                        }
                    }
                }
            }}
        </div>
    }
}

fn format_date(date_str: &str) -> String {
    if let Some(t_pos) = date_str.find('T') {
        date_str[..t_pos].to_string()
    } else {
        date_str.to_string()
    }
}
