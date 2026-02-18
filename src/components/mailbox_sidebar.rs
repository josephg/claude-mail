use crate::state::{AppState, MailView};
use jmap_client::Mailbox;
use leptos::prelude::*;

fn role_sort_order(role: Option<&str>) -> u32 {
    match role {
        Some("inbox") => 0,
        Some("drafts") => 1,
        Some("sent") => 2,
        Some("junk") => 3,
        Some("trash") => 4,
        Some(_) => 5,
        None => 6,
    }
}

/// Flatten the mailbox tree into a sorted list with depth info.
fn flatten_tree(mailboxes: &[Mailbox], parent_id: Option<&str>, depth: u32) -> Vec<(Mailbox, u32)> {
    let mut children: Vec<&Mailbox> = mailboxes
        .iter()
        .filter(|m| m.parent_id.as_deref() == parent_id)
        .collect();
    children.sort_by(|a, b| {
        role_sort_order(a.role.as_deref())
            .cmp(&role_sort_order(b.role.as_deref()))
            .then_with(|| a.sort_order.cmp(&b.sort_order))
            .then_with(|| a.name.cmp(&b.name))
    });

    let mut result = Vec::new();
    for child in children {
        let child_id = child.id.clone();
        result.push((child.clone(), depth));
        result.extend(flatten_tree(mailboxes, Some(&child_id), depth + 1));
    }
    result
}

#[component]
pub fn MailboxSidebar() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState to be provided");

    view! {
        <div class="mailbox-list">
            {move || {
                let mailboxes = state.mailboxes.get();
                let flat = flatten_tree(&mailboxes, None, 0);
                flat.into_iter().map(|(mailbox, depth)| {
                    let mailbox_id = mailbox.id.clone();
                    let mailbox_id_click = mailbox_id.clone();
                    let mailbox_name = mailbox.name.clone();
                    let unread = mailbox.unread_emails;
                    let padding_left = format!("{}px", depth * 16 + 8);

                    let on_click = move |_| {
                        state.selected_mailbox.set(Some(mailbox_id_click.clone()));
                        state.current_view.set(MailView::EmailList);
                        state.reply_to_email.set(None);
                        state.reply_all.set(false);
                    };

                    view! {
                        <div
                            class="mailbox-item"
                            class:active=move || state.selected_mailbox.get().as_deref() == Some(&mailbox_id)
                            style:padding-left=padding_left
                            on:click=on_click
                        >
                            <span class="mailbox-name">{mailbox_name}</span>
                            {if unread > 0 {
                                Some(view! { <span class="unread-badge">{unread}</span> })
                            } else {
                                None
                            }}
                        </div>
                    }
                }).collect_view()
            }}
        </div>
    }
}
