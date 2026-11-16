//! The Power page and its quick menu in a window, with sample data and no
//! system service touched:
//! `FLUX_POWER_PREVIEW=fr:laptop cargo run -p settings-power --example preview`
//! (`laptop` or `desktop`, and `:quick` for the quick menu alone).

use gtk4::prelude::*;

fn main() {
    let spec = std::env::var("FLUX_POWER_PREVIEW").unwrap_or_else(|_| "en:laptop".into());
    let mut parts = spec.split(':');
    let language = parts.next().unwrap_or("en").to_string();
    let laptop = parts.next() != Some("desktop");
    let quick = parts.next() == Some("quick");
    let application = gtk4::Application::builder()
        .application_id("studio.facile.FluxPowerPreview")
        .build();
    application.connect_activate(move |app| {
        shell_common::theme::init("window.preview { background-color: @fc_page; }", "settings-power-preview");
        settings_power::use_sample_data(&language, laptop);
        let window = gtk4::ApplicationWindow::new(app);
        window.add_css_class("preview");
        window.set_title(Some(&settings_power::title()));
        if quick {
            window.set_default_size(360, 160);
            window.set_child(Some(&settings_power::quick_menu()));
        } else {
            window.set_default_size(820, 900);
            window.set_child(Some(&settings_power::page()));
        }
        window.present();
    });
    let _ = shell_common::app::run(&application, "settings-power-preview");
}
