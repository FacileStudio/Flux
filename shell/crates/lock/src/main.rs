//! `flux-lock`: lock the session. See the crate README for the security model.

use lock::launch::{self, Mode};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = match launch::parse(&args) {
        Ok(mode) => mode,
        Err(unknown) => {
            eprintln!("flux-lock: unknown argument {unknown}\n{}", launch::USAGE);
            std::process::exit(2);
        }
    };
    let code = match mode {
        Mode::Version => {
            println!("flux-lock {}", env!("CARGO_PKG_VERSION"));
            0
        }
        Mode::Help => {
            print!("{}", launch::USAGE);
            0
        }
        Mode::Daemon => launch::launch(false),
        Mode::LockThenSuspend => launch::launch(true),
        Mode::Foreground => lock::run_locked(false),
        Mode::Notify => lock::run_locked(true),
    };
    std::process::exit(code);
}

