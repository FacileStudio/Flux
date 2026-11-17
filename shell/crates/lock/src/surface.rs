//! One lock surface: the widgets drawn on one output.

use gtk4::prelude::*;
use gtk4::{gdk, glib, Align, Orientation};

use crate::i18n::Text;

/// The widgets of one output's lock surface that change while it is up.
#[derive(Clone)]
pub struct Surface {
    pub root: gtk4::Overlay,
    pub content: gtk4::Box,
    pub background: gtk4::Picture,
    pub clock: gtk4::Label,
    pub date: gtk4::Label,
    pub hint: gtk4::Label,
    pub revealer: gtk4::Revealer,
    pub row: gtk4::Box,
    pub entry: gtk4::PasswordEntry,
    pub go: gtk4::Button,
    pub message: gtk4::Label,
    pub caps: gtk4::Label,
    pub layout: gtk4::Label,
    pub battery: gtk4::Label,
}

fn label(text: &str, class: &str) -> gtk4::Label {
    let label = gtk4::Label::new(Some(text));
    label.add_css_class(class);
    label
}

/// The password pill: the dots and the round arrow that submits them.
struct Field {
    column: gtk4::Box,
    row: gtk4::Box,
    entry: gtk4::PasswordEntry,
    go: gtk4::Button,
    message: gtk4::Label,
}

fn field(text: &Text, name: &str) -> Field {
    if let Some(settings) = gtk4::Settings::default() {
        settings.set_gtk_entry_select_on_focus(false);
    }
    let column = gtk4::Box::new(Orientation::Vertical, 12);
    column.set_halign(Align::Center);
    column.append(&label(name, "name"));
    let entry = gtk4::PasswordEntry::new();
    entry.set_placeholder_text(Some(text.placeholder));
    entry.set_show_peek_icon(false);
    entry.add_css_class("secret");
    entry.set_hexpand(true);
    let go = gtk4::Button::from_icon_name("go-next-symbolic");
    go.add_css_class("go");
    go.set_valign(Align::Center);
    go.set_tooltip_text(Some(text.unlock));
    let row = gtk4::Box::new(Orientation::Horizontal, 8);
    row.add_css_class("secret-row");
    row.set_width_request(340);
    row.append(&entry);
    row.append(&go);
    column.append(&row);
    let message = label("", "message");
    message.set_wrap(true);
    message.set_max_width_chars(40);
    message.set_justify(gtk4::Justification::Center);
    column.append(&message);
    Field { column, row, entry, go, message }
}

/// Build the surface for `name`, the user's display name.
pub fn build(text: &Text, name: &str) -> Surface {
    let background = gtk4::Picture::new();
    background.set_content_fit(gtk4::ContentFit::Cover);
    background.set_can_shrink(true);
    background.add_css_class("wallpaper");
    let root = gtk4::Overlay::new();
    root.add_css_class("lock");
    root.set_child(Some(&background));
    let scrim = gtk4::Box::new(Orientation::Vertical, 0);
    scrim.add_css_class("scrim");
    root.add_overlay(&scrim);

    let content = gtk4::Box::new(Orientation::Vertical, 0);
    content.add_css_class("content");
    let center = gtk4::Box::new(Orientation::Vertical, 8);
    center.set_valign(Align::Center);
    center.set_vexpand(true);
    let clock = label("", "clock");
    let date = label("", "date");
    center.append(&clock);
    center.append(&date);
    let hint = label(text.hint, "hint");
    center.append(&hint);
    let Field { column, row, entry, go, message } = field(text, name);
    let revealer = gtk4::Revealer::new();
    revealer.set_transition_type(gtk4::RevealerTransitionType::SlideUp);
    revealer.set_transition_duration(180);
    revealer.set_child(Some(&column));
    center.append(&revealer);
    content.append(&center);

    let footer = gtk4::Box::new(Orientation::Horizontal, 16);
    footer.add_css_class("footer");
    let caps = label(text.caps_lock, "caps");
    caps.set_visible(false);
    let layout = label("", "status");
    layout.set_visible(false);
    let battery = label("", "status");
    battery.set_visible(false);
    battery.set_hexpand(true);
    battery.set_halign(Align::End);
    footer.append(&caps);
    footer.append(&layout);
    footer.append(&battery);
    content.append(&footer);
    root.add_overlay(&content);

    Surface { root, content, background, clock, date, hint, revealer, row, entry, go, message, caps, layout, battery }
}

impl Surface {
    /// Show the password field, and add `first` to it when the key that
    /// revealed it typed a character. Keys pressed while the field slides in
    /// land here too, so they append rather than replace.
    pub fn reveal(&self, first: Option<char>) {
        self.revealer.set_reveal_child(true);
        self.hint.set_visible(false);
        self.entry.grab_focus();
        if let Some(character) = first {
            let mut typed = zeroize::Zeroizing::new(self.entry.text().to_string());
            typed.push(character);
            self.entry.set_text(&typed);
            self.entry.set_position(-1);
        }
    }

    /// Mark the field as refused, or clear that mark.
    pub fn set_error(&self, error: bool) {
        for widget in [self.entry.upcast_ref::<gtk4::Widget>(), self.row.upcast_ref()] {
            if error {
                widget.add_css_class("error");
            } else {
                widget.remove_css_class("error");
            }
        }
    }

    /// Whether keyboard focus is in the password field, which is a text widget inside the entry.
    pub fn typing(&self) -> bool {
        self.entry.root().and_then(|root| root.focus()).is_some_and(|focus| focus.is_ancestor(&self.entry))
    }

    /// Hide the field again and forget what was typed.
    pub fn conceal(&self) {
        self.entry.set_text("");
        self.message.set_text("");
        self.set_error(false);
        self.revealer.set_reveal_child(false);
        self.hint.set_visible(true);
    }

    /// Set the clock and date from `now`.
    pub fn tick(&self, now: &glib::DateTime) {
        if let Ok(time) = now.format("%H:%M") {
            self.clock.set_text(&time);
        }
        if let Ok(date) = now.format("%A %e %B") {
            self.date.set_text(&capitalize(&date.split_whitespace().collect::<Vec<_>>().join(" ")));
        }
    }

    /// Fade the content in once the surface is on screen.
    pub fn fade_in(&self) {
        let content = self.content.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(16), move || {
            content.add_css_class("shown");
        });
    }

    /// Draw `texture` behind the lock.
    pub fn set_background(&self, texture: &gdk::MemoryTexture) {
        self.background.set_paintable(Some(texture));
    }
}

/// `text` with its first letter in upper case, for French day names.
pub fn capitalize(text: &str) -> String {
    let mut characters = text.chars();
    match characters.next() {
        Some(first) => first.to_uppercase().chain(characters).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::capitalize;

    #[test]
    fn day_names_start_upper_case() {
        assert_eq!(capitalize("lundi 16 novembre"), "Lundi 16 novembre");
        assert_eq!(capitalize("élan"), "Élan");
        assert_eq!(capitalize(""), "");
    }
}
