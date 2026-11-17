//! The Wi-Fi list, keyed by SSID.
//!
//! A row that stays is updated and moved, never rebuilt, so the list reorders
//! without a flash. A new row slides open; a row that leaves slides shut and is
//! removed once its transition has run.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use gtk4::prelude::*;
use gtk4::{glib, Box, Orientation, Widget};

use super::row::NetRow;
use super::Page;
use crate::model::Network;
use crate::ui;

pub struct List {
    pub container: Box,
    rows: RefCell<HashMap<String, Rc<NetRow>>>,
}

impl List {
    pub fn new() -> List {
        let container = Box::new(Orientation::Vertical, 0);
        container.add_css_class("fx-list");
        List {
            container,
            rows: RefCell::new(HashMap::new()),
        }
    }

    /// Show `networks`, in that order.
    pub fn update(&self, page: &Rc<Page>, networks: &[Network]) {
        let mut rows = self.rows.borrow_mut();
        let keep: Vec<&str> = networks
            .iter()
            .map(|network| network.ssid.as_str())
            .collect();
        let gone: Vec<String> = rows
            .keys()
            .filter(|ssid| !keep.contains(&ssid.as_str()))
            .cloned()
            .collect();
        for ssid in gone {
            if let Some(row) = rows.remove(&ssid) {
                self.retire(&row);
            }
        }
        let mut previous: Option<Widget> = None;
        for network in networks {
            let row = rows
                .entry(network.ssid.clone())
                .or_insert_with(|| {
                    let row = NetRow::new(page, &network.ssid);
                    self.container.append(&row.outer);
                    row
                })
                .clone();
            row.update(page, network);
            self.container
                .reorder_child_after(&row.outer, previous.as_ref());
            row.outer.set_transition_duration(ui::motion(ui::MOTION_MS));
            row.outer.set_reveal_child(true);
            previous = Some(row.outer.clone().upcast());
        }
    }

    fn retire(&self, row: &NetRow) {
        row.outer.set_transition_duration(ui::motion(ui::MOTION_MS));
        row.outer.set_reveal_child(false);
        let outer = row.outer.clone();
        let container = self.container.clone();
        let delay = u64::from(ui::motion(ui::MOTION_MS)) + 40;
        glib::timeout_add_local_once(Duration::from_millis(delay), move || {
            if outer.parent().is_some_and(|parent| parent == container) {
                container.remove(&outer);
            }
        });
    }
}
