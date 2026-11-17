//! Running other programs without a shell, the run lock, and the progress a
//! page reads back: stdout lines, `/run/flux-config/state.json` and the log.

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use flux_config::paths::Paths;
use flux_config::progress::{Line, State, Step, Tracker};

/// The directories children search first, since `pkexec` resets `PATH`.
const SYSTEM_PATH: &str = "/run/wrappers/bin:/run/current-system/sw/bin";

/// How much this run may do.
#[derive(Clone, Debug)]
pub struct Mode {
    pub paths: Paths,
    pub dry_run: bool,
}

impl Mode {
    /// Whether programs such as `nixos-rebuild` really run: only on the real root without `--dry-run`.
    pub fn runs_commands(&self) -> bool {
        self.paths.is_system() && !self.dry_run
    }

    /// Whether files are written: always under a test root, never for a dry run on the real one.
    pub fn writes_files(&self) -> bool {
        !(self.dry_run && self.paths.is_system())
    }
}

/// Seconds since the epoch.
pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|age| age.as_secs()).unwrap_or(0)
}

/// Whether process `pid` is alive.
pub fn alive(pid: u32) -> bool {
    std::path::Path::new("/proc").join(pid.to_string()).exists()
}

/// The effective user id of this process.
pub fn euid() -> Option<u32> {
    let status = fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|line| line.starts_with("Uid:"))?;
    line.split_whitespace().nth(2)?.parse().ok()
}

/// Why the run lock was not taken.
pub enum LockError {
    Busy,
    Other(String),
}

impl From<String> for LockError {
    fn from(error: String) -> Self {
        LockError::Other(error)
    }
}

/// Hold `/run/flux-config/lock` for as long as the returned file lives.
pub fn lock(mode: &Mode) -> Result<Option<File>, LockError> {
    if !mode.writes_files() {
        return Ok(None);
    }
    let dir = mode.paths.run_dir();
    fs::create_dir_all(&dir).map_err(|error| format!("cannot create {}: {error}", dir.display()))?;
    let _ = fs::set_permissions(&dir, fs::Permissions::from_mode(0o755));
    let path = mode.paths.run_lock();
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(&path)
        .map_err(|error| format!("cannot open {}: {error}", path.display()))?;
    match file.try_lock() {
        Ok(()) => Ok(Some(file)),
        Err(std::fs::TryLockError::WouldBlock) => Err(LockError::Busy),
        Err(std::fs::TryLockError::Error(error)) => Err(format!("cannot lock {}: {error}", path.display()).into()),
    }
}

/// Where the progress of one operation goes.
pub struct Reporter {
    mode: Mode,
    operation: String,
    started: u64,
    log: Option<File>,
    pub restart: bool,
}

impl Reporter {
    /// Start reporting `operation`, emptying the log of the previous one.
    pub fn new(mode: &Mode, operation: &str) -> Reporter {
        let log = mode
            .writes_files()
            .then(|| {
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .mode(0o644)
                    .open(mode.paths.log())
                    .ok()
            })
            .flatten();
        let reporter =
            Reporter { mode: mode.clone(), operation: operation.to_string(), started: now(), log, restart: false };
        reporter.emit(&Line::new(Step::Start, 0, "Starting"));
        reporter
    }

    /// Print `line` on stdout and record it as the state a reopened page reads.
    pub fn emit(&self, line: &Line) {
        let mut stdout = std::io::stdout().lock();
        let _ = writeln!(stdout, "{}", line.to_json());
        let _ = stdout.flush();
        if !self.mode.writes_files() {
            return;
        }
        let state = State {
            operation: self.operation.clone(),
            pid: std::process::id(),
            started: self.started,
            line: line.clone(),
            restart: self.restart,
        };
        let path = self.mode.paths.state();
        let temp = path.with_extension("tmp");
        if fs::write(&temp, state.to_json()).is_ok() {
            let _ = fs::set_permissions(&temp, fs::Permissions::from_mode(0o644));
            let _ = fs::rename(&temp, &path);
        }
    }

    /// Emit `step` at `percent`.
    pub fn step(&self, step: Step, percent: u8, message: &str) {
        self.emit(&Line::new(step, percent, message));
    }

    /// Append one line of program output to the log.
    pub fn log(&mut self, text: &str) {
        if let Some(log) = self.log.as_mut() {
            let _ = writeln!(log, "{text}");
        }
    }
}

fn command(program: &str, args: &[String]) -> Command {
    let mut command = Command::new(program);
    command.args(args);
    let inherited = std::env::var("PATH").unwrap_or_default();
    command.env("PATH", format!("{SYSTEM_PATH}:{inherited}"));
    if std::env::var_os("HOME").is_none() {
        command.env("HOME", "/root");
    }
    command
}

fn shown(program: &str, args: &[String]) -> String {
    std::iter::once(program.to_string()).chain(args.iter().cloned()).collect::<Vec<_>>().join(" ")
}

/// Run `program` with its output merged into the log and read by `tracker`.
pub fn streamed(reporter: &mut Reporter, program: &str, args: &[String], tracker: &mut Tracker) -> Result<(), String> {
    reporter.log(&format!("$ {}", shown(program, args)));
    if !reporter.mode.runs_commands() {
        eprintln!("dry-run: {}", shown(program, args));
        return Ok(());
    }
    let (reader, writer) = std::io::pipe().map_err(|error| error.to_string())?;
    let error_writer = writer.try_clone().map_err(|error| error.to_string())?;
    let mut command = command(program, args);
    command.stdin(Stdio::null()).stdout(writer).stderr(error_writer);
    let mut child = command.spawn().map_err(|error| format!("cannot start {program}: {error}"))?;
    drop(command);
    let mut last = String::new();
    let lines = BufReader::new(reader).split(b'\n').map_while(Result::ok);
    for text in lines.map(|bytes| String::from_utf8_lossy(&bytes).into_owned()) {
        reporter.log(&text);
        if let Some(line) = tracker.feed(&text) {
            reporter.emit(&line);
        }
        if !text.trim().is_empty() {
            last = text;
        }
    }
    let status = child.wait().map_err(|error| format!("{program} did not finish: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{program} failed ({status}): {}", last.trim()))
    }
}

/// Run `program`, feed it `input`, and return its stdout. Stderr goes to the log, never `input`.
pub fn quiet(reporter: &mut Reporter, program: &str, args: &[String], input: Option<&[u8]>) -> Result<String, String> {
    reporter.log(&format!("$ {}", shown(program, args)));
    if !reporter.mode.runs_commands() {
        eprintln!("dry-run: {}{}", shown(program, args), if input.is_some() { " < (stdin)" } else { "" });
        return Ok(String::new());
    }
    let output = capture(program, args, input)?;
    reporter.log(output.1.trim_end());
    if output.2 {
        Ok(output.0)
    } else {
        Err(format!("{program} failed: {}", output.1.trim()))
    }
}

/// Run `program` and return its stdout, stderr and success, without a reporter.
pub fn capture(program: &str, args: &[String], input: Option<&[u8]>) -> Result<(String, String, bool), String> {
    let mut child = command(program, args)
        .stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("cannot start {program}: {error}"))?;
    if let (Some(input), Some(mut stdin)) = (input, child.stdin.take()) {
        stdin.write_all(input).map_err(|error| format!("cannot talk to {program}: {error}"))?;
    }
    let output = child.wait_with_output().map_err(|error| format!("{program} did not finish: {error}"))?;
    Ok((
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.success(),
    ))
}
