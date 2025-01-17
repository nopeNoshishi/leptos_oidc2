use components::App;
use leptos::prelude::*;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
mod components;
mod user;

fn main() {
    console_error_panic_hook::set_once();

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .without_time()
        .with_writer(tracing_web::MakeConsoleWriter)
        .pretty();

    tracing_subscriber::registry().with(fmt_layer).init();

    mount_to_body(|| view! { <App /> });
}
