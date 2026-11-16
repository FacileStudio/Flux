//! `flux-installer`: the full-screen Flux installer on the live ISO.

mod app;
mod i18n;
mod model;
mod system;
mod ui;

use gtk4::prelude::*;
use gtk4::Application;
use shell_common::app::run as run_app;

fn main() {
    if std::env::args().skip(1).any(|argument| argument == "--version") {
        println!("flux-installer {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let application = Application::builder()
        .application_id("studio.facile.FluxInstaller")
        .build();
    application.connect_activate(app::activate);
    let _ = run_app(&application, "flux-installer");
}
