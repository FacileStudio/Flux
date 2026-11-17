//! Text size, cursor size and motion, stored in `org.gnome.desktop.interface`
//! where GTK, the portal and the compositor read them.

use gtk4::gio;
use gtk4::prelude::*;
use shell_common::ui;

use crate::{i18n, preview};

const SCHEMA: &str = "org.gnome.desktop.interface";
const CURSOR_SIZES: [i32; 4] = [24, 32, 48, 64];

/// The interface settings, or `None` when the schema is not installed: `gio::Settings::new` aborts on a missing schema.
fn settings() -> Option<gio::Settings> {
    let schema = gio::SettingsSchemaSource::default()?.lookup(SCHEMA, true)?;
    Some(gio::Settings::new_full(
        &schema,
        None::<&gio::SettingsBackend>,
        None,
    ))
}

/// The index in [`CURSOR_SIZES`] closest to `size`.
pub fn cursor_index(size: i32) -> usize {
    CURSOR_SIZES
        .iter()
        .enumerate()
        .min_by_key(|(_, candidate)| (*candidate - size).abs())
        .map_or(0, |(index, _)| index)
}

/// The Text and motion section.
pub fn section() -> gtk4::Box {
    let text = i18n::text();
    let (root, card) = ui::section(text.interface);
    let settings = if preview::active() { None } else { settings() };
    if settings.is_none() && !preview::active() {
        card.append(&ui::empty_state(
            "preferences-desktop-appearance-symbolic",
            text.interface_missing,
            text.interface_missing_body,
        ));
        return root;
    }
    let scaling = settings
        .as_ref()
        .map_or(1.0, |settings| settings.double("text-scaling-factor"));
    let cursor = settings
        .as_ref()
        .map_or(24, |settings| settings.int("cursor-size"));
    let animations = settings
        .as_ref()
        .is_none_or(|settings| settings.boolean("enable-animations"));

    let store = settings.clone();
    let (line, _) = ui::slider_row(
        text.text_size,
        text.text_size_body,
        (0.8, 1.6, 0.05),
        scaling,
        move |value| {
            if let Some(settings) = &store {
                let _ = settings.set_double("text-scaling-factor", (value * 100.0).round() / 100.0);
            }
        },
    );
    card.append(&line);

    let store = settings.clone();
    let (line, _) = ui::choice_row(
        text.cursor_size,
        "",
        &text.cursor_sizes,
        cursor_index(cursor),
        move |index| {
            if let (Some(settings), Some(size)) = (&store, CURSOR_SIZES.get(index)) {
                let _ = settings.set_int("cursor-size", *size);
            }
        },
    );
    card.append(&line);

    let (line, _) = ui::switch_row(
        text.reduce_motion,
        text.reduce_motion_body,
        !animations,
        move |reduce| {
            if let Some(settings) = &settings {
                let _ = settings.set_boolean("enable-animations", !reduce);
            }
        },
    );
    card.append(&line);
    root
}

#[cfg(test)]
mod tests {
    use super::cursor_index;

    #[test]
    fn a_cursor_size_picks_the_nearest_choice() {
        assert_eq!(cursor_index(24), 0);
        assert_eq!(cursor_index(30), 1);
        assert_eq!(cursor_index(48), 2);
        assert_eq!(cursor_index(96), 3);
        assert_eq!(cursor_index(0), 0);
    }
}
