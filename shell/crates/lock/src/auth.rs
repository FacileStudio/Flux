//! The PAM conversation as a state machine, and the one decision that unlocks.
//!
//! Nothing in here talks to PAM. [`Backend`] is the seam: the real backend in
//! [`crate::pam`] drives a [`Conversation`] from libpam's callback, and the
//! tests drive it from a fake. [`verify`] is the only function whose result can
//! end the lock, and it returns [`Verdict::Unlock`] for a PAM success and for
//! nothing else.

use zeroize::Zeroizing;

#[cfg(test)]
mod tests;

/// One message PAM sends during a conversation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    Secret,
    Visible,
    Error,
    Info,
    Unknown,
}

/// What the conversation answers to one message.
#[derive(Debug, PartialEq, Eq)]
pub enum Reply {
    Secret,
    Text(String),
    Empty,
}

/// What a backend reports once PAM is done.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Success,
    Denied,
    Unavailable(String),
}

/// What the lock screen does next.
#[derive(Debug, PartialEq, Eq)]
pub enum Verdict {
    Unlock,
    Wrong(Option<String>),
    Unavailable(String),
}

/// A PAM implementation. The real one is libpam; tests use a fake.
pub trait Backend {
    fn authenticate(&self, user: &str, conversation: &mut Conversation) -> Outcome;
}

/// The answers one unlock attempt may give: the user name, the password once,
/// and nothing else.
///
/// A second password prompt (a new-password dance, a one-time code) fails the
/// conversation, so PAM returns an error and the session stays locked.
pub struct Conversation {
    user: String,
    secret: Zeroizing<Vec<u8>>,
    secret_sent: bool,
    messages: Vec<String>,
}

impl Conversation {
    /// A conversation that answers for `user` with `secret`.
    pub fn new(user: &str, secret: Zeroizing<Vec<u8>>) -> Conversation {
        Conversation {
            user: user.to_string(),
            secret,
            secret_sent: false,
            messages: Vec::new(),
        }
    }

    /// The answer to one PAM message, or `Err` to fail the conversation.
    pub fn respond(&mut self, style: Style, text: &str) -> Result<Reply, ()> {
        match style {
            Style::Secret if self.secret_sent => Err(()),
            Style::Secret => {
                self.secret_sent = true;
                Ok(Reply::Secret)
            }
            Style::Visible => Ok(Reply::Text(self.user.clone())),
            Style::Error | Style::Info => {
                let text = text.trim();
                if !text.is_empty() {
                    self.messages.push(text.to_string());
                }
                Ok(Reply::Empty)
            }
            Style::Unknown => Err(()),
        }
    }

    /// The password bytes, for the backend to copy into PAM's reply.
    pub fn secret(&self) -> &[u8] {
        &self.secret
    }

    /// The last message PAM wanted shown, if any.
    pub fn last_message(&self) -> Option<String> {
        self.messages.last().cloned()
    }
}

/// Check `password` for `user` through `backend`.
///
/// An empty password or one holding a NUL byte never reaches PAM: C cannot
/// carry the second, and the first is refused before the call so an account
/// without a password cannot be unlocked by pressing Enter.
pub fn verify(backend: &dyn Backend, user: &str, password: Zeroizing<String>) -> Verdict {
    if password.is_empty() || password.as_bytes().contains(&0) {
        return Verdict::Wrong(None);
    }
    let secret = Zeroizing::new(password.as_bytes().to_vec());
    drop(password);
    let mut conversation = Conversation::new(user, secret);
    match backend.authenticate(user, &mut conversation) {
        Outcome::Success => Verdict::Unlock,
        Outcome::Denied => Verdict::Wrong(conversation.last_message()),
        Outcome::Unavailable(reason) => Verdict::Unavailable(reason),
    }
}
