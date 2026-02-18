# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

This is a Rust WASM project using Trunk as the build tool:

- `trunk serve` — build and serve with live-reload at http://127.0.0.1:8080
- `trunk build` — production build to `dist/`
- `trunk build --release` — optimized release build
- `cargo check --target wasm32-unknown-unknown -p jmap-webmail` — fast type-check without full build
- `cargo check -p jmap-client` — check just the protocol library (native target, faster)

Requires `wasm32-unknown-unknown` target (`rustup target add wasm32-unknown-unknown`) and Trunk (`cargo install trunk`).

## Architecture

Two-crate workspace: a JMAP protocol client library and a Leptos CSR web frontend.

### jmap-client (library crate)

Pure JMAP protocol client, no browser dependencies. Uses reqwest (default-features disabled for WASM compatibility).

- `types.rs` — JMAP data types: Session, Mailbox, Email, EmailAddress, Thread, Identity, and protocol types (JmapRequest/JmapResponse/Invocation). Invocation has custom serde as a `[name, args, callId]` JSON array.
- `client.rs` — `JmapClient` struct. Connects via `/.well-known/jmap` with Basic auth. Methods: `get_mailboxes`, `query_emails`, `get_emails`, `get_thread`, `get_email_bodies`, `get_identities`, `send_email`. The `send_email` method creates a draft via `Email/set` and submits via `EmailSubmission/set` with `onSuccessUpdateEmail` to move from drafts to sent in a single API request.
- `error.rs` — `JmapError` enum.

### jmap-webmail (root crate, Leptos frontend)

CSR-only Leptos 0.7 app. No router — view switching is signal-driven via `AppState.current_view` (MailView enum) and `AppState.client` (None = login, Some = mail).

- `state.rs` — `AppState` (provided via `provide_context`, accessed via `use_context`). Contains RwSignals for client, mailboxes, selected mailbox, current view, identities, reply state. Also has localStorage helpers for credential persistence.
- `app.rs` — Root component. Checks localStorage for saved credentials and auto-connects on startup. Switches between LoginPage and MailPage based on whether client signal is set.
- `pages/login.rs` — Login form. On success, saves credentials to localStorage.
- `pages/mail.rs` — Mail shell: toolbar (Compose + Logout) + sidebar + right pane switcher.
- `components/mailbox_sidebar.rs` — Flattened tree rendering (avoids recursive `impl IntoView` which Rust can't resolve). Sorted by role priority then name.
- `components/email_list.rs` — Uses `LocalResource` to fetch emails when selected mailbox changes.
- `components/thread_view.rs` — Fetches thread via `Thread/get` then `Email/get` with body values.
- `components/compose.rs` — `ComposeView` (new email), `ComposeInline` (reply in thread), shared `ComposeForm`.

## Key Patterns

- **No router**: View state is in signals, not URLs. This avoids state loss on page refresh (credentials are restored from localStorage).
- **LocalResource for data fetching**: Reactive — re-fetches when tracked signals change. Returns `Option<SendWrapper<T>>`; access inner data via deref, not `.read()`.
- **spawn_local for fire-and-forget async**: Used in event handlers and login. Router hooks like `use_navigate` must be called during component setup (synchronous), not inside `spawn_local`.
- **into_any() for conditional rendering**: Required when match arms return different concrete view types.
- **CORS**: JMAP servers typically don't set CORS headers. Development requires a CORS proxy or server-side CORS configuration.
