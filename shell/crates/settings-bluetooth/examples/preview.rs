//! A window with only the Bluetooth page, for working on it outside Settings.
//!
//! `cargo run -p settings-bluetooth --example preview` drives the real BlueZ.
//! `FLUX_BLUETOOTH_PREVIEW=<scene>` shows sample data instead and touches no
//! service: devices, details, confirm, pin, display, service, pairing, off,
//! blocked, hardblocked, noadapter, noservice, or quick for the quick menu.

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};

fn main() {
    let application = Application::builder()
        .application_id("studio.facile.FluxBluetoothPreview")
        .build();
    application.connect_activate(|application| {
        let css: &'static str = Box::leak(format!("{}\nwindow {{ background-color: @fc_page; }}", shell_common::ui::CSS).into_boxed_str());
        shell_common::theme::init(css, "settings-bluetooth");
        let quick = std::env::var("FLUX_BLUETOOTH_PREVIEW").is_ok_and(|scene| scene == "quick");
        let content = if quick {
            settings_bluetooth::quick_menu()
        } else {
            settings_bluetooth::page()
        };
        let window = ApplicationWindow::builder()
            .application(application)
            .title(settings_bluetooth::title())
            .default_width(if quick { 340 } else { 760 })
            .default_height(if quick { 420 } else { 820 })
            .child(&content)
            .build();
        if quick {
            window.add_css_class("bt-quick-window");
        }
        window.present();
    });
    std::process::exit(shell_common::app::run(&application, "settings-bluetooth-preview").into());
}
