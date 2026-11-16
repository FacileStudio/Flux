//! Flux Settings pages for displays and input devices, both driven by niri.
//!
//! Each page writes a user-owned file that `/etc/niri/config.kdl` includes
//! (`~/.config/flux/niri/outputs.kdl` and `input.kdl`), and niri reloads it
//! live on every write.

mod backend;
pub mod display;
mod i18n;
pub mod input;
mod model;
mod ui;
