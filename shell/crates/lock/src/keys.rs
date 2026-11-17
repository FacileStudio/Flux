//! What a key press means to the lock screen, and where the keys typed while
//! a password is being checked wait.
//!
//! PAM takes about two seconds to refuse a wrong password. Someone who sees
//! the typo and types again at once would lose those keys, so they are held
//! in a [`KeyBuffer`] and put into the field when the answer is "wrong". On
//! "unlock" they are wiped without ever reaching the session.
//!
//! The buffer holds part of a password. It never grows, so the text is never
//! copied to a place that is not wiped; it is zeroed when taken, cleared or
//! dropped; it has no `Debug` and nothing here prints it.

use gtk4::gdk;
use zeroize::{Zeroize, Zeroizing};

#[cfg(test)]
mod tests;

/// The most bytes the buffer holds. Keys past this are dropped.
pub const CAPACITY: usize = 256;

/// What one key press types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Typed {
    Character(char),
    Erase,
    Forget,
    Nothing,
}

/// What `key` types with the modifiers in `state` held. A key pressed with
/// Control, Alt or Super is a shortcut and types nothing; so does a key with
/// no character of its own (Shift, an arrow, a dead key) and so does Enter.
pub fn typed(key: gdk::Key, state: gdk::ModifierType) -> Typed {
    let shortcut = gdk::ModifierType::CONTROL_MASK | gdk::ModifierType::ALT_MASK | gdk::ModifierType::SUPER_MASK;
    if key == gdk::Key::Escape {
        return Typed::Forget;
    }
    if state.intersects(shortcut) {
        return Typed::Nothing;
    }
    if key == gdk::Key::BackSpace {
        return Typed::Erase;
    }
    match key.to_unicode().filter(|character| !character.is_control()) {
        Some(character) => Typed::Character(character),
        None => Typed::Nothing,
    }
}

/// The keys typed while the lock is busy, in order.
pub struct KeyBuffer {
    typed: Zeroizing<String>,
}

impl KeyBuffer {
    pub fn new() -> KeyBuffer {
        KeyBuffer {
            typed: Zeroizing::new(String::with_capacity(CAPACITY)),
        }
    }

    /// Apply one key press. A character that does not fit is dropped whole.
    pub fn press(&mut self, typed: Typed) {
        match typed {
            Typed::Character(character) => {
                if self.typed.len() + character.len_utf8() <= CAPACITY {
                    self.typed.push(character);
                }
            }
            Typed::Erase => {
                self.typed.pop();
            }
            Typed::Forget => self.typed.zeroize(),
            Typed::Nothing => {}
        }
    }

    /// Move the held text out, leaving the buffer empty. The text wipes
    /// itself when the caller drops it.
    pub fn take(&mut self) -> Zeroizing<String> {
        std::mem::replace(&mut self.typed, Zeroizing::new(String::with_capacity(CAPACITY)))
    }
}

impl Default for KeyBuffer {
    fn default() -> KeyBuffer {
        KeyBuffer::new()
    }
}
