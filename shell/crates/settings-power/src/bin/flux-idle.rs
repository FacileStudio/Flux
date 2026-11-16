//! `flux-idle`: run swayidle with the rules from `~/.config/flux/power.json`.
//!
//! ```text
//! flux-idle run --swayidle PATH --lock PATH --niri PATH --inhibit PATH [--no-lock]
//! flux-idle suspend-if battery|ac
//! ```
//!
//! `run` replaces itself with swayidle, so the systemd unit supervises
//! swayidle directly. `suspend-if` is what the sleep timeouts call: it
//! suspends only when the machine is on the named power source, which is how
//! one swayidle carries a battery and a mains rule.

use std::os::unix::process::CommandExt;
use std::process::Command;

use settings_power::{config, idle};

const USAGE: &str = "usage: flux-idle run --swayidle PATH --lock PATH --niri PATH --inhibit PATH [--no-lock]\n       flux-idle suspend-if battery|ac";

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|arg| arg == name).and_then(|index| args.get(index + 1)).cloned()
}

fn run(args: &[String]) -> i32 {
    let (Some(swayidle), Some(lock), Some(niri), Some(inhibit)) = (
        flag(args, "--swayidle"),
        flag(args, "--lock"),
        flag(args, "--niri"),
        flag(args, "--inhibit"),
    ) else {
        eprintln!("{USAGE}");
        return 2;
    };
    let own = std::env::current_exe().map(|path| path.display().to_string()).unwrap_or_else(|_| "flux-idle".into());
    let programs = idle::Programs { swayidle, lock, niri, inhibit, idle: own };
    let lock_enabled = !args.iter().any(|arg| arg == "--no-lock");
    let command = idle::command(&config::load(), &programs, lock_enabled);
    let Some((program, rest)) = command.split_first() else {
        return 1;
    };
    let error = Command::new(program).args(rest).exec();
    eprintln!("flux-idle: cannot start {program}: {error}");
    1
}

fn suspend_if(source: Option<&str>) -> i32 {
    let mains = idle::on_mains(&idle::supplies());
    let wanted = match source {
        Some("battery") => !mains,
        Some("ac") => mains,
        _ => {
            eprintln!("{USAGE}");
            return 2;
        }
    };
    if !wanted {
        return 0;
    }
    match Command::new("systemctl").arg("suspend").status() {
        Ok(status) if status.success() => 0,
        _ => 1,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match args.first().map(String::as_str) {
        Some("run") => run(&args[1..]),
        Some("suspend-if") => suspend_if(args.get(1).map(String::as_str)),
        Some("--version") => {
            println!("flux-idle {}", env!("CARGO_PKG_VERSION"));
            0
        }
        _ => {
            eprintln!("{USAGE}");
            2
        }
    };
    std::process::exit(code);
}
