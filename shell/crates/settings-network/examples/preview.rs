//! A window with only the Network page, for trying it out:
//! `cargo run -p settings-network --example preview`, with
//! `FLUX_NETWORK_PREVIEW=fr:password` for sample data and `quick` as the first
//! argument for the quick menu.

use gtk4::prelude::*;

fn main() {
    let quick = std::env::args()
        .nth(1)
        .is_some_and(|argument| argument == "quick");
    let application = gtk4::Application::builder()
        .application_id("studio.facile.FluxNetworkPreview")
        .build();
    application.connect_activate(move |application| {
        shell_common::theme::init("", "settings-network");
        let window = gtk4::ApplicationWindow::builder()
            .application(application)
            .title(settings_network::title())
            .default_width(if quick { 360 } else { 760 })
            .default_height(if quick { 520 } else { 900 })
            .build();
        window.add_css_class("fx-network-preview");
        let child = if quick {
            settings_network::quick_menu()
        } else {
            settings_network::page()
        };
        window.set_child(Some(&child));
        window.present();
    });
    let _ = shell_common::app::run(&application, "settings-network-preview");
}
