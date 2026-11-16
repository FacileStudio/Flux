//! `flux-settings`: the Flux settings app. `flux-settings <id>` or
//! `flux-settings --page <id>` opens a page, and a second launch hands its page
//! to the window already open.

mod about;
mod app;
mod appearance;
mod i18n;
mod preview;
mod registry;

use gtk4::prelude::*;
use gtk4::{gio, glib, Application};

const APP_ID: &str = "studio.facile.flux.Settings";

/// The page named by `--page <id>`, `--page=<id>` or a bare `<id>` in `args`, program name first.
fn requested_page(args: &[String]) -> Option<String> {
    let mut rest = args.iter().skip(1);
    while let Some(argument) = rest.next() {
        if argument == "--page" {
            return rest.next().cloned();
        }
        if let Some(id) = argument.strip_prefix("--page=") {
            return Some(id.to_string());
        }
        if !argument.starts_with('-') {
            return Some(argument.clone());
        }
    }
    None
}

fn main() -> glib::ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().skip(1).any(|argument| argument == "--version") {
        println!("flux-settings {}", env!("CARGO_PKG_VERSION"));
        return glib::ExitCode::SUCCESS;
    }
    let mut flags = gio::ApplicationFlags::HANDLES_COMMAND_LINE;
    if let Some(preview) = preview::current() {
        i18n::force(&preview.language);
        preview::forward(preview);
        flags |= gio::ApplicationFlags::NON_UNIQUE;
    }
    let application = Application::builder()
        .application_id(APP_ID)
        .flags(flags)
        .build();
    application.connect_command_line(|application, line| {
        let args: Vec<String> = line
            .arguments()
            .iter()
            .filter_map(|argument| argument.to_str().map(str::to_string))
            .collect();
        let page = preview::current()
            .map(|preview| preview.page.clone())
            .or_else(|| requested_page(&args));
        app::open(application, page.as_deref());
        glib::ExitCode::SUCCESS
    });
    application.run_with_args(&args)
}

#[cfg(test)]
mod tests {
    use super::requested_page;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|item| item.to_string()).collect()
    }

    #[test]
    fn deep_links_name_a_page() {
        assert_eq!(
            requested_page(&args(&["flux-settings", "--page", "about"])),
            Some("about".into())
        );
        assert_eq!(
            requested_page(&args(&["flux-settings", "--page=appearance"])),
            Some("appearance".into())
        );
        assert_eq!(
            requested_page(&args(&["flux-settings", "network"])),
            Some("network".into())
        );
        assert_eq!(requested_page(&args(&["flux-settings"])), None);
        assert_eq!(requested_page(&args(&["flux-settings", "--page"])), None);
    }
}
