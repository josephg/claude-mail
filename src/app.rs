use crate::pages::{login::LoginPage, mail::MailPage};
use crate::state::AppState;
use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

#[component]
pub fn App() -> impl IntoView {
    let state = AppState::new();
    provide_context(state);

    view! {
        <Router>
            <Routes fallback=|| "Not found">
                <Route path=path!("/") view=LoginPage/>
                <Route path=path!("/mail") view=MailPage/>
            </Routes>
        </Router>
    }
}
