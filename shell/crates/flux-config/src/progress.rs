//! Progress lines, the state file that lets a reopened page reattach, and the
//! reading of `nixos-rebuild` output into friendly steps.
//!
//! Each line on stdout is one JSON object, as `flux-install --progress json`:
//! `{"step":"fetch|evaluate|build|activate|bootloader|clean|done","percent":0-100,"message":"..."}`,
//! and `{"step":"error","message":"..."}` before a non-zero exit.

use serde::{Deserialize, Serialize};

/// A stage of a long operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Step {
    Start,
    Fetch,
    Evaluate,
    Build,
    Activate,
    Bootloader,
    Clean,
    Password,
    Done,
    Error,
}

/// One progress line.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Line {
    pub step: Step,
    #[serde(default)]
    pub percent: u8,
    #[serde(default)]
    pub message: String,
}

impl Line {
    /// A line for `step` at `percent`.
    pub fn new(step: Step, percent: u8, message: impl Into<String>) -> Line {
        Line { step, percent: percent.min(100), message: message.into() }
    }

    /// The line as one JSON object, no newline.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Read one progress line; anything else is `None`.
    pub fn parse(text: &str) -> Option<Line> {
        serde_json::from_str(text.trim()).ok()
    }

    /// Whether the operation ended here.
    pub fn is_final(&self) -> bool {
        matches!(self.step, Step::Done | Step::Error)
    }
}

/// What `/run/flux-config/state.json` holds: the operation, who runs it, and its last line.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub operation: String,
    pub pid: u32,
    pub started: u64,
    pub line: Line,
    #[serde(default)]
    pub restart: bool,
}

impl State {
    /// Read a state document.
    pub fn parse(text: &str) -> Option<State> {
        serde_json::from_str(text).ok()
    }

    /// The document.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Whether the operation still runs, given whether its process is alive.
    pub fn running(&self, alive: bool) -> bool {
        alive && !self.line.is_final()
    }
}

/// Turns `nixos-rebuild` and `nix` output into progress lines, only when something changed.
#[derive(Debug)]
pub struct Tracker {
    step: Step,
    shown: u8,
    built: u32,
    copied: u32,
    base: u8,
}

impl Tracker {
    /// A tracker whose first step starts at `base` percent, so `update` can spend the first part on fetching.
    pub fn new(base: u8) -> Tracker {
        Tracker { step: Step::Start, shown: base, built: 0, copied: 0, base }
    }

    fn at(&mut self, step: Step, percent: u8, message: String) -> Option<Line> {
        if step == self.step && percent <= self.shown {
            return None;
        }
        self.step = step;
        self.shown = self.shown.max(percent);
        Some(Line::new(step, self.shown, message))
    }

    fn span(&self, from: u8, to: u8) -> u8 {
        let low = self.base.max(from);
        let done = u64::from(self.built + self.copied);
        let reach = u64::from(to.saturating_sub(low)) * done / (done + 200);
        low + u8::try_from(reach).unwrap_or(0)
    }

    /// Read one output line.
    pub fn feed(&mut self, text: &str) -> Option<Line> {
        match classify(text.trim())? {
            Step::Evaluate => {
                let percent = self.span(10, 10);
                self.at(Step::Evaluate, percent, "Preparing the new version".to_string())
            }
            Step::Fetch => {
                self.copied += 1;
                self.build()
            }
            Step::Build => {
                self.built += 1;
                self.build()
            }
            Step::Activate => self.at(Step::Activate, 88, "Switching to the new version".to_string()),
            step => self.at(step, 95, "Updating the boot menu".to_string()),
        }
    }

    fn build(&mut self) -> Option<Line> {
        let percent = self.span(15, 85);
        let message = format!("Downloading and building ({} downloaded, {} built)", self.copied, self.built);
        self.at(Step::Build, percent, message)
    }
}

const EVALUATE: &[&str] = &["building the system configuration", "evaluating"];
const BUILD: &[&str] = &["building '", "building \""];
const ACTIVATE: &[&str] =
    &["activating the configuration", "setting up /etc", "stopping the following units", "restarting the following units"];
const BOOTLOADER: &[&str] = &["updating GRUB", "Installing for"];

/// What kind of output line `text` is: `Fetch` stands for a path copied, `Build` for one built.
fn classify(text: &str) -> Option<Step> {
    let starts = |prefixes: &[&str]| prefixes.iter().any(|prefix| text.starts_with(prefix));
    if starts(EVALUATE) {
        Some(Step::Evaluate)
    } else if text.starts_with("copying path") {
        Some(Step::Fetch)
    } else if starts(BUILD) {
        Some(Step::Build)
    } else if starts(ACTIVATE) {
        Some(Step::Activate)
    } else if starts(BOOTLOADER) || text.contains("installing the boot loader") {
        Some(Step::Bootloader)
    } else {
        None
    }
}

/// Whether the running kernel differs from the active system's, from the two
/// `kernel` and `kernel-modules` link targets: booted first, current second.
pub fn needs_restart(booted: &[Option<String>], current: &[Option<String>]) -> bool {
    booted.iter().zip(current).any(|(a, b)| matches!((a, b), (Some(a), Some(b)) if a != b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_round_trip() {
        let line = Line::new(Step::Build, 40, "Downloading");
        assert_eq!(line.to_json(), r#"{"step":"build","percent":40,"message":"Downloading"}"#);
        assert_eq!(Line::parse(&line.to_json()), Some(line));
        assert_eq!(Line::parse(r#"{"step":"error","message":"boom"}"#), Some(Line::new(Step::Error, 0, "boom")));
        assert_eq!(Line::parse("copying path '/nix/store/x'"), None);
        assert_eq!(Line::parse(r#"{"step":"warp","percent":1}"#), None);
        assert_eq!(Line::new(Step::Done, 250, "").percent, 100);
    }

    #[test]
    fn tracker_moves_forward_only() {
        let mut tracker = Tracker::new(0);
        let lines: Vec<Line> = [
            "building the system configuration...",
            "these 3 derivations will be built:",
            "copying path '/nix/store/a' from 'https://cache.nixos.org'...",
            "copying path '/nix/store/b' from 'https://cache.nixos.org'...",
            "building '/nix/store/c.drv'...",
            "activating the configuration...",
            "setting up /etc...",
            "updating GRUB 2 menu...",
        ]
        .iter()
        .filter_map(|text| tracker.feed(text))
        .collect();
        let mut steps: Vec<Step> = lines.iter().map(|line| line.step).collect();
        steps.dedup();
        assert_eq!(steps, [Step::Evaluate, Step::Build, Step::Activate, Step::Bootloader]);
        assert!(lines.windows(2).all(|pair| pair[0].percent <= pair[1].percent));
        assert_eq!(lines.last().map(|line| line.percent), Some(95));
    }

    #[test]
    fn tracker_counts_paths() {
        let mut tracker = Tracker::new(0);
        let last = (0..300)
            .filter_map(|index| tracker.feed(&format!("copying path '/nix/store/{index}' from 'https://cache'...")))
            .last();
        let last = last.unwrap_or_else(|| Line::new(Step::Error, 0, ""));
        assert_eq!(last.step, Step::Build);
        assert!(last.percent > 50 && last.percent < 85, "{}", last.percent);
        assert!(last.message.contains("downloaded, 0 built"));
    }

    #[test]
    fn tracker_respects_base() {
        let mut tracker = Tracker::new(30);
        let line = tracker.feed("building the system configuration...");
        assert_eq!(line.map(|line| line.percent), Some(30));
    }

    #[test]
    fn state_round_trip() {
        let state = State {
            operation: "update".to_string(),
            pid: 42,
            started: 1_700_000_000,
            line: Line::new(Step::Build, 50, "Downloading"),
            restart: false,
        };
        assert_eq!(State::parse(&state.to_json()), Some(state.clone()));
        assert!(state.running(true));
        assert!(!state.running(false));
    }

    #[test]
    fn restart_detection() {
        let a = Some("/nix/store/a-linux-6.12".to_string());
        let b = Some("/nix/store/b-linux-6.13".to_string());
        assert!(needs_restart(std::slice::from_ref(&a), std::slice::from_ref(&b)));
        assert!(!needs_restart(std::slice::from_ref(&a), std::slice::from_ref(&a)));
        assert!(!needs_restart(&[None], &[b]));
    }
}
