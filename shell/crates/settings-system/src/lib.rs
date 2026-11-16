//! The System, Users and Updates pages of Flux Settings. Each page follows the
//! Settings contract (`page`, `ID`, `ICON`, `title`) and talks to the
//! `flux-config` helper for anything NixOS owns.

mod backend;
mod i18n;
mod task;
mod ui;

pub mod system;
pub mod updates;
pub mod users;

/// The stylesheet the pages need on top of the shared Settings sheet.
pub const CSS: &str = include_str!("../style.css");
