//! What each choice of the exit menu runs.

/// The choices, in the order the menu shows them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Lock,
    Suspend,
    Logout,
    Restart,
    PowerOff,
}

pub const ALL: [Action; 5] = [
    Action::Lock,
    Action::Suspend,
    Action::Logout,
    Action::Restart,
    Action::PowerOff,
];

impl Action {
    /// The command lines to try, in order, until one starts.
    pub fn commands(self) -> Vec<Vec<String>> {
        let line = |parts: &[&str]| parts.iter().map(|part| part.to_string()).collect::<Vec<_>>();
        match self {
            Action::Lock => vec![line(&["flux-lock"]), line(&["loginctl", "lock-session"])],
            Action::Suspend => vec![line(&["systemctl", "suspend"])],
            Action::Logout => {
                let mut lines = vec![line(&["niri", "msg", "action", "quit", "--skip-confirmation"])];
                if let Some(session) = std::env::var("XDG_SESSION_ID").ok().filter(|id| !id.is_empty()) {
                    lines.push(line(&["loginctl", "terminate-session", &session]));
                }
                lines
            }
            Action::Restart => vec![line(&["systemctl", "reboot"])],
            Action::PowerOff => vec![line(&["systemctl", "poweroff"])],
        }
    }

    /// The symbolic icon the menu draws for the choice.
    pub fn icon(self) -> &'static str {
        match self {
            Action::Lock => "system-lock-screen-symbolic",
            Action::Suspend => "media-playback-pause-symbolic",
            Action::Logout => "system-log-out-symbolic",
            Action::Restart => "system-reboot-symbolic",
            Action::PowerOff => "system-shutdown-symbolic",
        }
    }

    /// The CSS class that tells the choices apart.
    pub fn class(self) -> &'static str {
        match self {
            Action::Lock => "lock",
            Action::Suspend => "suspend",
            Action::Logout => "logout",
            Action::Restart => "restart",
            Action::PowerOff => "poweroff",
        }
    }
}

/// Run the first of `commands` that starts, and report whether one did.
pub fn run(commands: &[Vec<String>]) -> bool {
    commands.iter().any(|command| {
        let Some((program, args)) = command.split_first() else {
            return false;
        };
        std::process::Command::new(program)
            .args(args)
            .stdin(std::process::Stdio::null())
            .spawn()
            .is_ok()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_out_quits_niri_first() {
        let commands = Action::Logout.commands();
        assert_eq!(commands[0], ["niri", "msg", "action", "quit", "--skip-confirmation"]);
        assert!(commands.iter().all(|command| command[0] != "swaymsg"));
    }

    #[test]
    fn every_choice_has_a_command_an_icon_and_a_class() {
        for action in ALL {
            assert!(!action.commands().is_empty(), "{action:?}");
            assert!(action.icon().ends_with("-symbolic"), "{action:?}");
            assert!(!action.class().is_empty(), "{action:?}");
        }
        assert_eq!(Action::Lock.commands()[0], ["flux-lock"]);
        assert_eq!(Action::Suspend.commands()[0], ["systemctl", "suspend"]);
        assert_eq!(Action::Restart.commands()[0], ["systemctl", "reboot"]);
        assert_eq!(Action::PowerOff.commands()[0], ["systemctl", "poweroff"]);
    }

    #[test]
    fn running_nothing_reports_failure() {
        assert!(!run(&[]));
        assert!(!run(&[vec!["surely-not-a-program-xyz".to_string()]]));
        assert!(run(&[vec!["surely-not-a-program-xyz".to_string()], vec!["true".to_string()]]));
    }
}
