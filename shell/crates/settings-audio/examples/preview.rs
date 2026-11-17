//! Open the Sound page, or the quick menu, in a window.
//!
//! `cargo run -p settings-audio --example preview` talks to the session's sound server.
//! `FLUX_AUDIO_PREVIEW=1` swaps it for sample data and touches no service; add `quick` as the
//! first argument for the quick menu. `LANG=fr_FR.UTF-8` shows the French strings.

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};

fn main() {
    let quick = std::env::args()
        .nth(1)
        .is_some_and(|argument| argument == "quick");
    let application = Application::builder()
        .application_id("studio.facile.FluxSoundPreview")
        .build();
    application.connect_activate(move |application| {
        shell_common::theme::init(
            Box::leak(format!("{}\nwindow.preview {{ background-color: @fc_page; }}", shell_common::ui::CSS).into_boxed_str()),
            "flux-settings",
        );
        let window = ApplicationWindow::builder()
            .application(application)
            .title(settings_audio::title())
            .default_width(if quick { 360 } else { 760 })
            .default_height(if quick { 420 } else { 900 })
            .build();
        window.add_css_class("preview");
        let child = if quick {
            settings_audio::quick_menu()
        } else {
            settings_audio::page()
        };
        window.set_child(Some(&child));
        window.present();
    });
    let _ = shell_common::app::run(&application, "settings-audio-preview");
}
