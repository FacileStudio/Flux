//! Open the lock surface in a plain window with sample data, for screenshots:
//! `FLUX_LOCK_PREVIEW=fr:wrong cargo run -p lock --example lock-preview`.

fn main() {
    let spec = std::env::var("FLUX_LOCK_PREVIEW").unwrap_or_else(|_| "en:idle".into());
    std::process::exit(lock::preview(&spec));
}
