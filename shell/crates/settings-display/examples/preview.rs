//! Open one page in a window with sample data, touching no system service:
//! `cargo run -p settings-display --example preview -- display` or `input`.
//! `FLUX_LANG=fr` shows the French strings.

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};
use settings_display::{display, input};

const WINDOW_CSS: &str = "window.fx-preview { background-color: @fc_page; }";

fn main() {
    if std::env::var_os("FLUX_SETTINGS_PREVIEW").is_none() {
        std::env::set_var("FLUX_SETTINGS_PREVIEW", "1");
    }
    let which = std::env::args().nth(1).unwrap_or_else(|| display::ID.to_string());
    let application = Application::builder().application_id("studio.facile.FluxSettingsPreview").build();
    application.connect_activate(move |application| {
        shell_common::theme::init(WINDOW_CSS, "flux-settings-preview");
        let (title, page) = if which == input::ID {
            (input::title(), input::page())
        } else {
            (display::title(), display::page())
        };
        let window = ApplicationWindow::builder()
            .application(application)
            .title(title)
            .default_width(860)
            .default_height(900)
            .css_classes(["fx-preview"])
            .child(&page)
            .build();
        window.present();
    });
    let _ = shell_common::app::run(&application, "settings-display-preview");
}
