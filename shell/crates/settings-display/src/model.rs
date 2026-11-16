//! Everything the pages compute without touching GTK or the system.

pub mod config;
pub mod geometry;
pub mod night;
pub mod outputs;
pub mod xkb;

#[cfg(test)]
mod tests;
