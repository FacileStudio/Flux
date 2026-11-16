//! The libpam backend: `pam_start("flux-lock")`, `pam_authenticate`, `pam_end`.
//!
//! Written against the Linux-PAM ABI directly instead of through a crate: the
//! PAM crates on crates.io generate their bindings with bindgen at build time,
//! which pulls libclang into the build for five functions and two structs.
//!
//! The conversation callback never panics across the C boundary, answers only
//! through [`Conversation::respond`], and hands PAM replies allocated with
//! `calloc`/`malloc` as `pam_conv(3)` requires: PAM owns and frees them.

use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::auth::{Backend, Conversation, Outcome, Reply, Style};

const SERVICE: &str = "flux-lock";

const PAM_SUCCESS: c_int = 0;
const PAM_BUF_ERR: c_int = 5;
const PAM_CONV_ERR: c_int = 19;
const PAM_AUTH_ERR: c_int = 7;
const PAM_USER_UNKNOWN: c_int = 10;
const PAM_MAXTRIES: c_int = 11;
const PAM_CRED_INSUFFICIENT: c_int = 8;
const PAM_DISALLOW_NULL_AUTHTOK: c_int = 0x1;

const PAM_PROMPT_ECHO_OFF: c_int = 1;
const PAM_PROMPT_ECHO_ON: c_int = 2;
const PAM_ERROR_MSG: c_int = 3;
const PAM_TEXT_INFO: c_int = 4;

#[repr(C)]
struct PamMessage {
    msg_style: c_int,
    msg: *const c_char,
}

#[repr(C)]
struct PamResponse {
    resp: *mut c_char,
    resp_retcode: c_int,
}

type ConvFn = unsafe extern "C" fn(
    c_int,
    *mut *const PamMessage,
    *mut *mut PamResponse,
    *mut c_void,
) -> c_int;

#[repr(C)]
struct PamConv {
    conv: Option<ConvFn>,
    appdata_ptr: *mut c_void,
}

#[link(name = "pam")]
extern "C" {
    fn pam_start(
        service: *const c_char,
        user: *const c_char,
        conv: *const PamConv,
        handle: *mut *mut c_void,
    ) -> c_int;
    fn pam_authenticate(handle: *mut c_void, flags: c_int) -> c_int;
    fn pam_end(handle: *mut c_void, status: c_int) -> c_int;
}

/// Authentication through the system's PAM stack for the `flux-lock` service.
///
/// Every pointer handed to libpam (service, user, conversation) outlives the
/// transaction, which `pam_end` closes before any of them is dropped, and the
/// handle is used only after a successful `pam_start`.
pub struct LibPam;

impl Backend for LibPam {
    fn authenticate(&self, user: &str, conversation: &mut Conversation) -> Outcome {
        let (Ok(service), Ok(name)) = (CString::new(SERVICE), CString::new(user)) else {
            return Outcome::Unavailable("invalid user name".into());
        };
        let conv = PamConv {
            conv: Some(converse),
            appdata_ptr: (conversation as *mut Conversation).cast(),
        };
        let mut handle: *mut c_void = std::ptr::null_mut();
        let started = unsafe { pam_start(service.as_ptr(), name.as_ptr(), &conv, &mut handle) };
        if started != PAM_SUCCESS || handle.is_null() {
            return Outcome::Unavailable(format!("pam_start failed ({started})"));
        }
        let status = unsafe { pam_authenticate(handle, PAM_DISALLOW_NULL_AUTHTOK) };
        unsafe { pam_end(handle, status) };
        outcome(status)
    }
}

/// What a `pam_authenticate` status means for the lock.
fn outcome(status: c_int) -> Outcome {
    match status {
        PAM_SUCCESS => Outcome::Success,
        PAM_AUTH_ERR | PAM_USER_UNKNOWN | PAM_MAXTRIES | PAM_CRED_INSUFFICIENT => Outcome::Denied,
        other => Outcome::Unavailable(format!("pam_authenticate failed ({other})")),
    }
}

fn style(code: c_int) -> Style {
    match code {
        PAM_PROMPT_ECHO_OFF => Style::Secret,
        PAM_PROMPT_ECHO_ON => Style::Visible,
        PAM_ERROR_MSG => Style::Error,
        PAM_TEXT_INFO => Style::Info,
        _ => Style::Unknown,
    }
}

/// Copy `bytes` into a NUL-terminated `malloc` buffer that PAM will free.
///
/// The buffer is `len + 1` bytes and fully written before it is returned.
fn c_copy(bytes: &[u8]) -> *mut c_char {
    unsafe {
        let buffer = libc::malloc(bytes.len() + 1).cast::<u8>();
        if buffer.is_null() {
            return std::ptr::null_mut();
        }
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer, bytes.len());
        *buffer.add(bytes.len()) = 0;
        buffer.cast()
    }
}

/// Wipe and free the replies built so far, after a failed conversation.
///
/// # Safety
/// `replies` holds `count` entries allocated by [`converse`].
unsafe fn discard(replies: *mut PamResponse, count: usize) {
    for index in 0..count {
        let reply = &mut *replies.add(index);
        if !reply.resp.is_null() {
            let length = libc::strlen(reply.resp);
            std::ptr::write_bytes(reply.resp, 0, length);
            libc::free(reply.resp.cast());
        }
    }
    libc::free(replies.cast());
}

/// The `pam_conv` callback.
///
/// # Safety
/// Called by libpam with `count` message pointers and the `Conversation`
/// pointer given to `pam_start` as `data`.
unsafe extern "C" fn converse(
    count: c_int,
    messages: *mut *const PamMessage,
    replies: *mut *mut PamResponse,
    data: *mut c_void,
) -> c_int {
    let answer = catch_unwind(AssertUnwindSafe(|| {
        let Ok(count) = usize::try_from(count) else {
            return PAM_CONV_ERR;
        };
        if count == 0 || messages.is_null() || replies.is_null() || data.is_null() {
            return PAM_CONV_ERR;
        }
        let conversation = &mut *data.cast::<Conversation>();
        let out = libc::calloc(count, std::mem::size_of::<PamResponse>()).cast::<PamResponse>();
        if out.is_null() {
            return PAM_BUF_ERR;
        }
        for index in 0..count {
            let message = &**messages.add(index);
            let text = if message.msg.is_null() {
                String::new()
            } else {
                CStr::from_ptr(message.msg).to_string_lossy().into_owned()
            };
            let resp = match conversation.respond(style(message.msg_style), &text) {
                Err(()) => {
                    discard(out, index);
                    return PAM_CONV_ERR;
                }
                Ok(Reply::Secret) => c_copy(conversation.secret()),
                Ok(Reply::Text(value)) => c_copy(value.as_bytes()),
                Ok(Reply::Empty) => std::ptr::null_mut(),
            };
            (*out.add(index)).resp = resp;
        }
        *replies = out;
        PAM_SUCCESS
    }));
    answer.unwrap_or(PAM_CONV_ERR)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_pam_success_is_success() {
        assert_eq!(outcome(PAM_SUCCESS), Outcome::Success);
        assert_eq!(outcome(PAM_AUTH_ERR), Outcome::Denied);
        assert_eq!(outcome(PAM_MAXTRIES), Outcome::Denied);
        assert!(matches!(outcome(PAM_CONV_ERR), Outcome::Unavailable(_)));
        assert!(matches!(outcome(4), Outcome::Unavailable(_)));
    }

    #[test]
    fn message_styles_map_and_unknown_ones_fail() {
        assert_eq!(style(PAM_PROMPT_ECHO_OFF), Style::Secret);
        assert_eq!(style(PAM_PROMPT_ECHO_ON), Style::Visible);
        assert_eq!(style(PAM_ERROR_MSG), Style::Error);
        assert_eq!(style(PAM_TEXT_INFO), Style::Info);
        assert_eq!(style(99), Style::Unknown);
    }
}
