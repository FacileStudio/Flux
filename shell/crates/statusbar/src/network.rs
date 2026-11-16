//! Network pill for the statusbar: the SSID or the connection state on the bar,
//! and a detail popup on click. The probing and the wording live in [`status`]
//! and [`text`]; the probes run subprocesses, so they only run off the main loop.

mod link;
mod status;
mod text;

use gtk4::prelude::*;

use crate::popup::{self, Popup};
use crate::work;

/// The pill's label, or `None` when no manager and no interface can answer.
pub fn label() -> Option<String> {
    status::state().map(|state| text::label(&state, crate::text::get()))
}

/// Open the network popup once the state is read.
pub fn show(popup: &Popup) {
    let popup = popup.clone();
    work::off_main(status::state, move |state| {
        let strings = crate::text::get();
        let Some(state) = state else {
            popup::message(&popup, strings.network, strings.unavailable);
            return;
        };
        let summary = text::label(&state, strings);
        let detail = text::detail(&state, strings);
        popup.show(move |pop| {
            pop.append(&popup::header(strings.network));
            pop.append(&popup::value(&summary));
            pop.append(&popup::detail(&detail));
        });
    });
}
