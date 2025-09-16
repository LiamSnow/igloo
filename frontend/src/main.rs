mod editor;
mod frontend;

use leptos::prelude::*;
use frontend::App;

fn main() {
    leptos::mount::mount_to_body(|| {
        view! {
            <p>"Hello, world!"</p>
            <App />
        }
    })
}
