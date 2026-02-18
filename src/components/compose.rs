use crate::state::{AppState, MailView};
use jmap_client::EmailAddress;
use leptos::prelude::*;
use leptos::task::spawn_local;

/// Full-pane compose view for new emails.
#[component]
pub fn ComposeView() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState to be provided");

    // Pre-fill from current_view state
    let (init_to, init_cc, init_bcc, init_subject, init_body) = match state.current_view.get() {
        MailView::Compose {
            to,
            cc,
            bcc,
            subject,
            body,
        } => (to, cc, bcc, subject, body),
        _ => Default::default(),
    };

    let on_cancel = move |_| {
        state.current_view.set(MailView::EmailList);
    };

    let on_sent = move || {
        state.current_view.set(MailView::EmailList);
    };

    view! {
        <div class="compose-view">
            <h2>"New Message"</h2>
            <ComposeForm
                initial_to=init_to
                initial_cc=init_cc
                initial_bcc=init_bcc
                initial_subject=init_subject
                initial_body=init_body
                on_cancel=on_cancel
                on_sent=on_sent
            />
        </div>
    }
}

/// Inline compose for replies within the thread view.
#[component]
pub fn ComposeInline(email_id: String) -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState to be provided");
    let is_reply_all = state.reply_all.get();

    // Find the email being replied to and pre-fill fields
    let (init_to, init_cc, init_subject, init_body) = {
        let client = state.client.get();
        let identities = state.identities.get();
        let my_email = identities.first().map(|i| i.email.clone()).unwrap_or_default();

        // We need to look up this email. For now, use a simple approach:
        // the thread view already loaded the emails, so we'll fetch again.
        // In a real app, we'd cache this.
        let email_id_clone = email_id.clone();
        let (to, cc, subject, body) = if let Some(client) = client {
            // We'll set defaults; actual pre-fill happens async
            let _ = (client, email_id_clone);
            (String::new(), String::new(), String::new(), String::new())
        } else {
            (String::new(), String::new(), String::new(), String::new())
        };

        // Prefill will be done in a resource below
        let _ = (my_email, is_reply_all);
        (to, cc, subject, body)
    };

    let to_signal = RwSignal::new(init_to);
    let cc_signal = RwSignal::new(init_cc);
    let subject_signal = RwSignal::new(init_subject);
    let body_signal = RwSignal::new(init_body);

    // Async prefill from email data
    let email_id_fetch = email_id.clone();
    spawn_local(async move {
        let client = state.client.get();
        let Some(client) = client else { return };
        let identities = state.identities.get();
        let my_email = identities.first().map(|i| i.email.clone()).unwrap_or_default();

        let emails = client
            .get_email_bodies(&[email_id_fetch])
            .await
            .ok()
            .unwrap_or_default();
        let Some(email) = emails.first() else { return };

        // Build subject
        let orig_subject = email.subject.as_deref().unwrap_or("");
        let re_subject = if orig_subject.starts_with("Re: ") {
            orig_subject.to_string()
        } else {
            format!("Re: {orig_subject}")
        };
        subject_signal.set(re_subject);

        // Build To
        let from_addrs = email
            .from
            .as_ref()
            .map(|a| {
                a.iter()
                    .map(|addr| addr.email.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();

        if is_reply_all {
            // To = original From + original To (minus self)
            let mut to_list: Vec<String> = Vec::new();
            if let Some(from) = &email.from {
                to_list.extend(from.iter().map(|a| a.email.clone()));
            }
            if let Some(to) = &email.to {
                to_list.extend(
                    to.iter()
                        .filter(|a| a.email != my_email)
                        .map(|a| a.email.clone()),
                );
            }
            to_signal.set(to_list.join(", "));

            // Cc = original Cc
            let cc_list = email
                .cc
                .as_ref()
                .map(|addrs| {
                    addrs
                        .iter()
                        .filter(|a| a.email != my_email)
                        .map(|a| a.email.clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            cc_signal.set(cc_list);
        } else {
            to_signal.set(from_addrs);
        }

        // Build body with quoted text
        let orig_from = email
            .from
            .as_ref()
            .and_then(|a| a.first())
            .map(|a| a.to_string())
            .unwrap_or_default();
        let orig_date = email.received_at.as_deref().unwrap_or("");
        let orig_body = email
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
            .unwrap_or_default();

        let quoted = orig_body
            .lines()
            .map(|line| format!("> {line}"))
            .collect::<Vec<_>>()
            .join("\n");

        body_signal.set(format!(
            "\n\nOn {orig_date}, {orig_from} wrote:\n{quoted}"
        ));
    });

    let on_cancel = move |_| {
        state.reply_to_email.set(None);
    };

    let on_sent = move || {
        state.reply_to_email.set(None);
    };

    view! {
        <div class="compose-inline">
            <ComposeForm
                initial_to=to_signal.get()
                initial_cc=cc_signal.get()
                initial_bcc=String::new()
                initial_subject=subject_signal.get()
                initial_body=body_signal.get()
                on_cancel=on_cancel
                on_sent=on_sent
            />
        </div>
    }
}

/// Shared compose form used by both ComposeView and ComposeInline.
#[component]
fn ComposeForm(
    initial_to: String,
    initial_cc: String,
    initial_bcc: String,
    initial_subject: String,
    initial_body: String,
    on_cancel: impl Fn(leptos::ev::MouseEvent) + 'static,
    on_sent: impl Fn() + Clone + 'static,
) -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState to be provided");

    let to = RwSignal::new(initial_to);
    let cc = RwSignal::new(initial_cc);
    let bcc = RwSignal::new(initial_bcc);
    let subject = RwSignal::new(initial_subject);
    let body = RwSignal::new(initial_body);
    let sending = RwSignal::new(false);
    let error_msg = RwSignal::new(Option::<String>::None);

    let on_sent_clone = on_sent.clone();
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let to_val = to.get();
        let cc_val = cc.get();
        let bcc_val = bcc.get();
        let subject_val = subject.get();
        let body_val = body.get();
        let on_sent = on_sent_clone.clone();

        sending.set(true);
        error_msg.set(None);

        spawn_local(async move {
            let client = state.client.get();
            let Some(client) = client else {
                error_msg.set(Some("Not connected".to_string()));
                sending.set(false);
                return;
            };

            let identities = state.identities.get();
            let Some(identity) = identities.first() else {
                error_msg.set(Some("No identity found".to_string()));
                sending.set(false);
                return;
            };

            let mailboxes = state.mailboxes.get();
            let drafts_id = client
                .find_mailbox_by_role(&mailboxes, "drafts")
                .map(|m| m.id.clone())
                .unwrap_or_default();
            let sent_id = client
                .find_mailbox_by_role(&mailboxes, "sent")
                .map(|m| m.id.clone())
                .unwrap_or_default();

            let from_addrs = vec![EmailAddress {
                name: identity.name.clone(),
                email: identity.email.clone(),
            }];
            let to_addrs = parse_addresses(&to_val);
            let cc_addrs = parse_addresses(&cc_val);
            let bcc_addrs = parse_addresses(&bcc_val);

            match client
                .send_email(
                    &identity.id,
                    &from_addrs,
                    &to_addrs,
                    &cc_addrs,
                    &bcc_addrs,
                    &subject_val,
                    &body_val,
                    &drafts_id,
                    &sent_id,
                )
                .await
            {
                Ok(()) => {
                    on_sent();
                }
                Err(e) => {
                    error_msg.set(Some(format!("Send failed: {e}")));
                    sending.set(false);
                }
            }
        });
    };

    view! {
        <form class="compose-form" on:submit=on_submit>
            <div class="form-field">
                <label>"To"</label>
                <input type="text" bind:value=to placeholder="recipient@example.com"/>
            </div>
            <div class="form-field">
                <label>"Cc"</label>
                <input type="text" bind:value=cc/>
            </div>
            <div class="form-field">
                <label>"Bcc"</label>
                <input type="text" bind:value=bcc/>
            </div>
            <div class="form-field">
                <label>"Subject"</label>
                <input type="text" bind:value=subject/>
            </div>
            <div class="form-field">
                <label>"Body"</label>
                <textarea rows="12" bind:value=body></textarea>
            </div>
            <div class="compose-actions">
                <button type="submit" disabled=move || sending.get()>
                    {move || if sending.get() { "Sending..." } else { "Send" }}
                </button>
                <button type="button" on:click=on_cancel>"Cancel"</button>
            </div>
            {move || error_msg.get().map(|msg| view! {
                <div class="error-message">{msg}</div>
            })}
        </form>
    }
}

fn parse_addresses(input: &str) -> Vec<EmailAddress> {
    input
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| EmailAddress {
            name: None,
            email: s.to_string(),
        })
        .collect()
}
