use crate::router::mailbox_id_to_slug;
use crate::state::AppState;
use jmap_client::Email;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use wasm_bindgen::JsCast;

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

const PAGE_SIZE: u64 = 50;

#[component]
pub fn EmailList() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState to be provided");
    let navigate = use_navigate();

    let emails: RwSignal<Vec<Email>> = RwSignal::new(vec![]);
    let has_more = RwSignal::new(false);
    let loading = RwSignal::new(false);

    let load_page = move |position: u64, append: bool| {
        loading.set(true);
        spawn_local(async move {
            let client = state.client.get_untracked();
            let mailbox_id = state.selected_mailbox.get_untracked();
            let (Some(client), Some(mailbox_id)) = (client, mailbox_id) else {
                loading.set(false);
                return;
            };
            let (ids, total) = client
                .query_emails(&mailbox_id, position, PAGE_SIZE)
                .await
                .ok()
                .unwrap_or_default();
            if ids.is_empty() {
                has_more.set(false);
                loading.set(false);
                return;
            }
            let (new_emails, email_state) = client
                .get_emails(&ids, LIST_PROPERTIES)
                .await
                .ok()
                .unwrap_or_default();
            state.email_state.set(Some(email_state));

            let loaded_count = new_emails.len() as u64;
            if append {
                emails.update(|list| list.extend(new_emails));
            } else {
                emails.set(new_emails);
            }
            has_more.set((position + loaded_count) < total);
            loading.set(false);
        });
    };

    // Reset and load when mailbox or refresh trigger changes
    Effect::new(move || {
        let _mailbox = state.selected_mailbox.get();
        let _refresh = state.email_refresh_trigger.get();
        let _client = state.client.get();
        emails.set(vec![]);
        has_more.set(false);
        load_page(0, false);
    });

    // Scroll handler for infinite scroll
    let on_scroll = move |ev: web_sys::Event| {
        if loading.get_untracked() || !has_more.get_untracked() {
            return;
        }
        let Some(target) = ev.target() else { return };
        let el: web_sys::Element = target.unchecked_into();
        let at_bottom = el.scroll_top() + el.client_height() >= el.scroll_height() - 200;
        if at_bottom {
            let position = emails.with_untracked(|list| list.len() as u64);
            load_page(position, true);
        }
    };

    view! {
        <div class="email-list" on:scroll=on_scroll>
            <For
                each=move || emails.get()
                key=|email| email.id.clone().unwrap_or_default()
                children=move |email| {
                    let thread_id = email.thread_id.clone().unwrap_or_default();
                    let subject = email.subject.clone().unwrap_or_else(|| "(no subject)".to_string());
                    let preview = email.preview.clone().unwrap_or_default();
                    let from = email.from.as_ref()
                        .and_then(|addrs| addrs.first())
                        .map(|a| a.name.as_deref().unwrap_or(&a.email).to_string())
                        .unwrap_or_else(|| "(unknown)".to_string());
                    let date = email.received_at.clone().unwrap_or_default();
                    let is_unread = !email.keywords.as_ref().map_or(false, |kw| kw.contains_key("$seen"));
                    let has_attachment = email.has_attachment.unwrap_or(false);
                    let nav = navigate.clone();

                    let on_click = {
                        let thread_id = thread_id.clone();
                        move |_| {
                            let mailboxes = state.mailboxes.get();
                            let slug = state
                                .selected_mailbox
                                .get()
                                .as_deref()
                                .map(|id| mailbox_id_to_slug(&mailboxes, id))
                                .unwrap_or_else(|| "inbox".to_string());
                            nav(
                                &format!("/mail/{slug}/{}", thread_id),
                                Default::default(),
                            );
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
                }
            />
            {move || {
                let list_empty = emails.with(|l| l.is_empty());
                let is_loading = loading.get();
                if list_empty && is_loading {
                    Some(view! { <div class="loading">"Loading..."</div> })
                } else if list_empty {
                    Some(view! { <div class="empty">"No emails in this mailbox"</div> })
                } else if is_loading {
                    Some(view! { <div class="loading-more">"Loading more..."</div> })
                } else {
                    None
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
