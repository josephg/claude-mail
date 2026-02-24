# JMAP Webmail

A webmail client built on the [JMAP](https://jmap.io/) protocol (RFC 8620 / RFC 8621), using Rust compiled to WebAssembly.

**[Live demo](https://seph.au/claude-webmail/)** (requires a JMAP mail server to connect to)

This is a **purely client-side application** — there is no server component. The compiled WASM/HTML/CSS can be served as static files from any web server. It connects directly to a JMAP-compliant mail server such as [Stalwart](https://stalw.art/), [Fastmail](https://www.fastmail.com/), [Cyrus](https://www.cyrusimap.org/), or any other server implementing the JMAP standard.

This code was almost entirely written by Claude Opus 4.6 (Anthropic's AI coding assistant) via [Claude Code](https://claude.ai/code).

## Features

- Login with Basic auth via JMAP's `.well-known/jmap` autodiscovery
- Mailbox sidebar with nested folder tree and unread counts
- Email list with infinite scroll
- Threaded conversation view
- Compose new emails, reply, and reply-all
- Real-time push notifications via JMAP EventSource (SSE)
- Credential persistence in localStorage with auto-login
- URL-based routing (`/mail/inbox`, `/mail/sent/THREAD_ID`, etc.)

## Architecture

Two-crate Rust workspace:

- **`jmap-client`** — Pure JMAP protocol client library. Handles session discovery, mailbox/email/thread queries, email submission, and state change parsing. No browser dependencies; uses reqwest with default-features disabled for WASM compatibility.
- **`jmap-webmail`** (root crate) — [Leptos](https://leptos.dev/) 0.8 CSR frontend compiled to WASM via [Trunk](https://trunkrs.dev/). Client-side rendered single-page app with `leptos_router` for URL routing.

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- WASM target: `rustup target add wasm32-unknown-unknown`
- [Trunk](https://trunkrs.dev/): `cargo install trunk`

## Development

```sh
trunk serve
```

Builds and serves at http://127.0.0.1:8080 with live-reload.

### Other commands

```sh
trunk build                  # dev build to dist/
trunk build --release        # optimized release build

# Fast type-checking without full WASM build:
cargo check --target wasm32-unknown-unknown -p jmap-webmail
cargo check -p jmap-client   # library only (native target, faster)
```

### Deploying under a subpath

The app can be deployed at any URL path, not just the root. Set `BASE_URL` and `--public-url` when building:

```sh
BASE_URL=/claude-webmail trunk build --release --public-url /claude-webmail/
```

This configures both the router (so routes resolve relative to the subpath) and Trunk's asset paths.

## CORS

JMAP servers typically don't set CORS headers. You'll need either:

- A CORS proxy in front of the JMAP server
- Server-side CORS configuration allowing your origin

## URL Structure

| URL | View |
|-----|------|
| `/` | Redirects to `/mail/inbox` |
| `/login` | Login page |
| `/mail/:mailbox` | Email list |
| `/mail/:mailbox/compose` | Compose new email |
| `/mail/:mailbox/:thread_id` | Thread view |

Mailbox slugs use the JMAP role name for well-known folders (`inbox`, `drafts`, `sent`, `junk`, `trash`, `archive`) and the raw JMAP mailbox ID otherwise.

## License

ISC
