use crate::state::AppState;
use crate::sync::handle_state_change;
use jmap_client::StateChange;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

/// Replace `{types}`, `{closeafter}`, `{ping}` in the EventSource URL template.
fn expand_event_source_url(template: &str, types: &str, closeafter: &str, ping: &str) -> String {
    template
        .replace("{types}", types)
        .replace("{closeafter}", closeafter)
        .replace("{ping}", ping)
}

/// Line-based SSE parser.
struct SseParser {
    event_type: String,
    data: String,
}

impl SseParser {
    fn new() -> Self {
        Self {
            event_type: String::new(),
            data: String::new(),
        }
    }

    /// Feed a single line. Returns Some((event_type, data)) when a complete event is ready.
    fn feed_line(&mut self, line: &str) -> Option<(String, String)> {
        if line.is_empty() {
            // Blank line = dispatch event
            if !self.data.is_empty() {
                let event_type = if self.event_type.is_empty() {
                    "message".to_string()
                } else {
                    std::mem::take(&mut self.event_type)
                };
                let data = std::mem::take(&mut self.data);
                return Some((event_type, data));
            }
            self.event_type.clear();
            return None;
        }

        if line.starts_with(':') {
            // Comment, ignore
            return None;
        }

        if let Some(value) = line.strip_prefix("event:") {
            self.event_type = value.trim().to_string();
        } else if let Some(value) = line.strip_prefix("data:") {
            if !self.data.is_empty() {
                self.data.push('\n');
            }
            self.data.push_str(value.trim());
        } else if let Some(_value) = line.strip_prefix("id:") {
            // We don't track last-event-id for reconnection
        }

        None
    }
}

pub fn start_event_source(state: AppState) {
    spawn_local(async move {
        event_source_loop(state).await;
    });
}

async fn event_source_loop(state: AppState) {
    loop {
        let Some(client) = state.client.get_untracked() else {
            return;
        };

        let Some(url_template) = client.session().event_source_url.as_deref() else {
            web_sys::console::log_1(&"No eventSourceUrl in session, push disabled".into());
            return;
        };

        let url = expand_event_source_url(url_template, "*", "no", "30");
        let auth = client.auth_header().to_string();

        // Drop client ref before entering the streaming loop
        drop(client);

        match open_sse_stream(&url, &auth, state).await {
            Ok(()) => {
                // Stream ended cleanly (e.g. server closed connection)
            }
            Err(e) => {
                web_sys::console::warn_1(
                    &format!("EventSource error: {e:?}, reconnecting in 3s...").into(),
                );
            }
        }

        // Check if still logged in before reconnecting
        if state.client.get_untracked().is_none() {
            return;
        }

        // Wait 3 seconds before reconnecting
        let promise = js_sys::Promise::new(&mut |resolve, _| {
            let _ = web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 3000);
        });
        let _ = JsFuture::from(promise).await;

        // Re-check after sleep
        if state.client.get_untracked().is_none() {
            return;
        }
    }
}

async fn open_sse_stream(
    url: &str,
    auth: &str,
    state: AppState,
) -> Result<(), JsValue> {
    let abort_controller = web_sys::AbortController::new()?;
    let abort_signal = abort_controller.signal();
    state.sse_abort.set_value(Some(abort_controller));

    let opts = web_sys::RequestInit::new();
    opts.set_method("GET");
    opts.set_signal(Some(&abort_signal));

    let request = web_sys::Request::new_with_str_and_init(url, &opts)?;
    request.headers().set("Authorization", auth)?;
    request.headers().set("Accept", "text/event-stream")?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: web_sys::Response = resp_value.dyn_into()?;

    if !resp.ok() {
        return Err(JsValue::from_str(&format!(
            "SSE fetch failed with status {}",
            resp.status()
        )));
    }

    let body = resp.body().ok_or_else(|| JsValue::from_str("No body in SSE response"))?;
    let reader: web_sys::ReadableStreamDefaultReader = body.get_reader().dyn_into()?;

    let mut parser = SseParser::new();
    let mut leftover = String::new();

    loop {
        let result = JsFuture::from(reader.read()).await?;
        let done = js_sys::Reflect::get(&result, &"done".into())?
            .as_bool()
            .unwrap_or(true);

        if done {
            break;
        }

        let value = js_sys::Reflect::get(&result, &"value".into())?;
        let chunk = js_sys::Uint8Array::new(&value);
        let bytes = chunk.to_vec();
        let text = String::from_utf8_lossy(&bytes);

        leftover.push_str(&text);

        // Process complete lines
        while let Some(newline_pos) = leftover.find('\n') {
            let line = leftover[..newline_pos].trim_end_matches('\r').to_string();
            leftover = leftover[newline_pos + 1..].to_string();

            if let Some((event_type, data)) = parser.feed_line(&line) {
                if event_type == "state" {
                    if let Ok(change) = serde_json::from_str::<StateChange>(&data) {
                        handle_state_change(state, change);
                    }
                }
            }
        }

        // Check if logged out
        if state.client.get_untracked().is_none() {
            let _ = reader.cancel();
            break;
        }
    }

    Ok(())
}
