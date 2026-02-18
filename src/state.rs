use jmap_client::{Identity, JmapClient, Mailbox};
use leptos::prelude::*;
use web_sys::window;

const STORAGE_KEY: &str = "jmap_credentials";

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

    pub fn logout(&self) {
        self.client.set(None);
        self.mailboxes.set(vec![]);
        self.selected_mailbox.set(None);
        self.current_view.set(MailView::EmailList);
        self.identities.set(vec![]);
        self.reply_to_email.set(None);
        self.reply_all.set(false);
        clear_saved_credentials();
    }
}

pub fn save_credentials(server: &str, username: &str, password: &str) {
    let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten()) else {
        return;
    };
    let json = serde_json::json!({
        "server": server,
        "username": username,
        "password": password,
    });
    let _ = storage.set_item(STORAGE_KEY, &json.to_string());
}

pub fn load_saved_credentials() -> Option<(String, String, String)> {
    let storage = window().and_then(|w| w.local_storage().ok().flatten())?;
    let raw = storage.get_item(STORAGE_KEY).ok().flatten()?;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let server = v["server"].as_str()?.to_string();
    let username = v["username"].as_str()?.to_string();
    let password = v["password"].as_str()?.to_string();
    Some((server, username, password))
}

pub fn clear_saved_credentials() {
    if let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.remove_item(STORAGE_KEY);
    }
}
