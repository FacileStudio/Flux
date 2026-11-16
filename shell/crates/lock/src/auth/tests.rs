use std::cell::{Cell, RefCell};

use zeroize::Zeroizing;

use super::*;

struct Fake {
    script: Vec<(Style, &'static str)>,
    password: &'static str,
    calls: Cell<u32>,
    seen_user: RefCell<String>,
    broken: bool,
}

impl Fake {
    fn new(script: Vec<(Style, &'static str)>, password: &'static str) -> Fake {
        Fake {
            script,
            password,
            calls: Cell::new(0),
            seen_user: RefCell::new(String::new()),
            broken: false,
        }
    }
}

impl Backend for Fake {
    fn authenticate(&self, _user: &str, conversation: &mut Conversation) -> Outcome {
        self.calls.set(self.calls.get() + 1);
        if self.broken {
            return Outcome::Unavailable("no pam".into());
        }
        let mut good = false;
        for (style, text) in &self.script {
            match conversation.respond(*style, text) {
                Err(()) => return Outcome::Unavailable("conversation".into()),
                Ok(Reply::Secret) => good = conversation.secret() == self.password.as_bytes(),
                Ok(Reply::Text(name)) => *self.seen_user.borrow_mut() = name,
                Ok(Reply::Empty) => {}
            }
        }
        if good {
            Outcome::Success
        } else {
            Outcome::Denied
        }
    }
}

fn secret(text: &str) -> Zeroizing<String> {
    Zeroizing::new(text.to_string())
}

#[test]
fn the_right_password_unlocks() {
    let fake = Fake::new(vec![(Style::Secret, "Password: ")], "hunter2");
    assert_eq!(verify(&fake, "ada", secret("hunter2")), Verdict::Unlock);
}

#[test]
fn a_wrong_password_stays_locked_and_carries_the_pam_message() {
    let fake = Fake::new(
        vec![(Style::Secret, "Password: "), (Style::Error, " Sorry ")],
        "hunter2",
    );
    assert_eq!(
        verify(&fake, "ada", secret("nope")),
        Verdict::Wrong(Some("Sorry".into()))
    );
}

#[test]
fn a_second_password_prompt_fails_closed() {
    let fake = Fake::new(
        vec![(Style::Secret, "Password: "), (Style::Secret, "New password: ")],
        "hunter2",
    );
    assert!(matches!(
        verify(&fake, "ada", secret("hunter2")),
        Verdict::Unavailable(_)
    ));
}

#[test]
fn an_unknown_message_style_fails_closed() {
    let fake = Fake::new(vec![(Style::Unknown, ""), (Style::Secret, "")], "hunter2");
    assert!(matches!(
        verify(&fake, "ada", secret("hunter2")),
        Verdict::Unavailable(_)
    ));
}

#[test]
fn a_broken_backend_stays_locked() {
    let mut fake = Fake::new(vec![], "hunter2");
    fake.broken = true;
    assert_eq!(
        verify(&fake, "ada", secret("hunter2")),
        Verdict::Unavailable("no pam".into())
    );
}

#[test]
fn empty_and_nul_passwords_never_reach_pam() {
    let fake = Fake::new(vec![(Style::Secret, "")], "");
    assert_eq!(verify(&fake, "ada", secret("")), Verdict::Wrong(None));
    assert_eq!(verify(&fake, "ada", secret("a\0b")), Verdict::Wrong(None));
    assert_eq!(fake.calls.get(), 0);
}

#[test]
fn a_visible_prompt_gets_the_user_name() {
    let fake = Fake::new(
        vec![(Style::Visible, "login: "), (Style::Info, ""), (Style::Secret, "")],
        "hunter2",
    );
    assert_eq!(verify(&fake, "ada", secret("hunter2")), Verdict::Unlock);
    assert_eq!(*fake.seen_user.borrow(), "ada");
}

#[test]
fn success_without_a_password_prompt_is_pams_call() {
    struct Permit;
    impl Backend for Permit {
        fn authenticate(&self, _: &str, _: &mut Conversation) -> Outcome {
            Outcome::Success
        }
    }
    assert_eq!(verify(&Permit, "ada", secret("x")), Verdict::Unlock);
}
