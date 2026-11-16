//! The statusbar's power popover: one battery line and the mode control.

use gtk4::prelude::*;
use gtk4::Orientation;

use crate::format;
use crate::i18n::Text;
use crate::modes::Modes;
use crate::source::Source;

fn icon_for(percentage: f64, charging: bool) -> String {
    let step = ((percentage.clamp(0.0, 100.0) / 10.0).round() as u32) * 10;
    if charging {
        format!("battery-level-{step}-charging-symbolic")
    } else {
        format!("battery-level-{step}-symbolic")
    }
}

/// Build the quick menu from `source`.
pub fn build(text: &'static Text, source: Source) -> gtk4::Widget {
    let column = gtk4::Box::new(Orientation::Vertical, 12);
    column.add_css_class("fx-quick");
    column.set_margin_top(12);
    column.set_margin_bottom(12);
    column.set_margin_start(12);
    column.set_margin_end(12);
    let line = gtk4::Box::new(Orientation::Horizontal, 8);
    let icon = gtk4::Image::new();
    let label = gtk4::Label::new(None);
    label.add_css_class("fx-title");
    label.set_xalign(0.0);
    line.append(&icon);
    line.append(&label);
    column.append(&line);
    let modes = Modes::new(text, &source);
    column.append(&modes.root);
    let sync = move |source: &Source| {
        let snapshot = source.snapshot();
        line.set_visible(snapshot.battery.is_some());
        if let Some(battery) = &snapshot.battery {
            let charging = battery.state == crate::upower::State::Charging;
            icon.set_icon_name(Some(&icon_for(battery.percentage, charging)));
            label.set_text(&format!("{} · {}", format::percent(text, battery.percentage), format::battery_line(text, battery)));
        }
        modes.sync(&snapshot);
    };
    sync(&source);
    source.on_change(move |source| sync(source));
    column.upcast()
}

#[cfg(test)]
mod tests {
    use super::icon_for;

    #[test]
    fn icons_step_by_ten() {
        assert_eq!(icon_for(76.0, false), "battery-level-80-symbolic");
        assert_eq!(icon_for(4.0, true), "battery-level-0-charging-symbolic");
        assert_eq!(icon_for(100.0, false), "battery-level-100-symbolic");
    }
}
