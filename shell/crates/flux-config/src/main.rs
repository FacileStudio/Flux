//! `flux-config`: the root helper behind Settings. It edits
//! `/etc/flux/settings.json`, rebuilds the system, updates, lists and rolls
//! back generations, and sets passwords. Run it through
//! `pkexec flux-config ...`; read-only commands work as any user.

mod exec;
mod ops;

use std::process::ExitCode;

use exec::Mode;
use flux_config::paths::Paths;

const USAGE: &str = "usage: flux-config [--dry-run] <command>

read-only:
  get                              settings.json (or what the system says) as JSON
  status                           the running or last operation, restart needed, version
  check                            whether a newer Flux exists on the followed branch
  generations                      the system versions, newest first

changes (root, through pkexec; each applies unless --no-apply is given):
  set hostname <name>
  set timezone <zone|auto>
  set locale <locale>
  set formats <locale|same>
  set keymap <layout>
  set auto-update <on|off>         no rebuild needed
  set <key> <value> [<key> <value>]...   several at once, one rebuild
  user add <name> --full-name <text> [--admin]   password on stdin
  user remove <name>
  user rename <name> <full name>   changes the name people see, never the login
  user set-admin <name> <on|off>
  user set-password <name>         password on stdin, no rebuild
  apply                            nixos-rebuild switch --flake /etc/flux#flux
  update [--scheduled]             move the flux input to its latest revision, then apply
  rollback [generation]            switch to an older generation
  gc [--keep N]                    delete all but the N newest generations (default 3)

FLUX_CONFIG_ROOT=<dir> moves every path under <dir> and never runs a program.
--dry-run prints the programs instead of running them and writes nothing to /.";

/// The parsed command line.
pub struct Args {
    pub words: Vec<String>,
    pub apply: bool,
    pub admin: bool,
    pub scheduled: bool,
    pub full_name: Option<String>,
    pub keep: Option<String>,
}

fn parse(raw: Vec<String>) -> Result<(Args, bool), String> {
    let mut args = Args { words: Vec::new(), apply: true, admin: false, scheduled: false, full_name: None, keep: None };
    let mut dry_run = false;
    let mut iter = raw.into_iter();
    while let Some(word) = iter.next() {
        match word.as_str() {
            "--dry-run" => dry_run = true,
            "--no-apply" => args.apply = false,
            "--admin" => args.admin = true,
            "--scheduled" => args.scheduled = true,
            "--full-name" => args.full_name = Some(iter.next().ok_or("--full-name needs a value")?),
            "--keep" => args.keep = Some(iter.next().ok_or("--keep needs a value")?),
            "--" => args.words.extend(iter.by_ref()),
            flag if flag.starts_with("--") => return Err(format!("unknown option {flag}")),
            _ => args.words.push(word),
        }
    }
    Ok((args, dry_run))
}

fn main() -> ExitCode {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    if raw.iter().any(|word| word == "--version") {
        println!("flux-config {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    if raw.is_empty() || raw.iter().any(|word| word == "-h" || word == "--help") {
        println!("{USAGE}");
        return if raw.is_empty() { ExitCode::from(2) } else { ExitCode::SUCCESS };
    }
    let (args, dry_run) = match parse(raw) {
        Ok(parsed) => parsed,
        Err(error) => {
            eprintln!("flux-config: {error}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let mode = Mode { paths: Paths::from_env(), dry_run };
    match ops::run(&mode, &args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(ops::Failure::Usage(error)) => {
            eprintln!("flux-config: {error}\n\n{USAGE}");
            ExitCode::from(2)
        }
        Err(ops::Failure::Failed(error)) => {
            eprintln!("flux-config: {error}");
            ExitCode::FAILURE
        }
    }
}
