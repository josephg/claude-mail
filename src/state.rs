use jmap_client::{Identity, JmapClient, Mailbox};
use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum MailView {
    EmailList,
    ThreadView { thread_id: String },
    Compose {
        to: String,
        cc: String,
        bcc: String,
        subject: String,
        body: String,
    },
}

#[derive(Clone, Copy)]
pub struct AppState {
    pub client: RwSignal<Option<JmapClient>>,
    pub mailboxes: RwSignal<Vec<Mailbox>>,
    pub selected_mailbox: RwSignal<Option<String>>,
    pub current_view: RwSignal<MailView>,
    pub identities: RwSignal<Vec<Identity>>,
    pub reply_to_email: RwSignal<Option<String>>,
    pub reply_all: RwSignal<bool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            client: RwSignal::new(None),
            mailboxes: RwSignal::new(vec![]),
            selected_mailbox: RwSignal::new(None),
            current_view: RwSignal::new(MailView::EmailList),
            identities: RwSignal::new(vec![]),
            reply_to_email: RwSignal::new(None),
            reply_all: RwSignal::new(false),
        }
    }
}
