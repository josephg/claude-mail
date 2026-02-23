use jmap_client::{Identity, JmapClient, Mailbox};
use leptos::prelude::*;
use web_sys::window;

const STORAGE_KEY: &str = "jmap_credentials";

#[derive(Clone, Copy)]
pub struct AppState {
    pub client: RwSignal<Option<JmapClient>>,
    pub mailboxes: RwSignal<Vec<Mailbox>>,
    pub selected_mailbox: RwSignal<Option<String>>,
    pub identities: RwSignal<Vec<Identity>>,
    pub reply_to_email: RwSignal<Option<String>>,
    pub reply_all: RwSignal<bool>,
    pub email_state: RwSignal<Option<String>>,
    pub mailbox_state: RwSignal<Option<String>>,
    pub email_refresh_trigger: RwSignal<u64>,
    pub auto_login_done: RwSignal<bool>,
    pub sse_abort: StoredValue<Option<web_sys::AbortController>, LocalStorage>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            client: RwSignal::new(None),
            mailboxes: RwSignal::new(vec![]),
            selected_mailbox: RwSignal::new(None),
            identities: RwSignal::new(vec![]),
            reply_to_email: RwSignal::new(None),
            reply_all: RwSignal::new(false),
            email_state: RwSignal::new(None),
            mailbox_state: RwSignal::new(None),
            email_refresh_trigger: RwSignal::new(0),
            auto_login_done: RwSignal::new(false),
            sse_abort: StoredValue::new_local(None),
        }
    }

    pub fn logout(&self) {
        // Abort SSE connection before clearing client
        self.sse_abort.with_value(|v| {
            if let Some(controller) = v {
                controller.abort();
            }
        });
        self.sse_abort.set_value(None);
        self.client.set(None);
        self.mailboxes.set(vec![]);
        self.selected_mailbox.set(None);
        self.identities.set(vec![]);
        self.reply_to_email.set(None);
        self.reply_all.set(false);
        self.email_state.set(None);
        self.mailbox_state.set(None);
        self.email_refresh_trigger.set(0);
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
