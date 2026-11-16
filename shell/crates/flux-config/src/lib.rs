//! What `flux-config` decides without touching the system: input validation,
//! the `settings.json` model, progress lines, generation lists and the flake
//! lock. The `settings-system` pages reuse it so both sides agree on every rule.

pub mod flake;
pub mod generations;
pub mod machine;
pub mod paths;
pub mod progress;
pub mod seed;
pub mod settings;
pub mod validate;
