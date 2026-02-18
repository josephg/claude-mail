mod app;
mod components;
mod eventsource;
mod pages;
mod state;
mod sync;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(app::App);
}
