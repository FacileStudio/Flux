//! Running a blocking read off the GTK main loop and handing its result back.

use gtk4::{gio, glib};

/// Run `work` on GIO's blocking pool, then `done` on the main loop with what it
/// returned. A worker that panics drops its result and `done` never runs.
pub fn off_main<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
    done: impl FnOnce(T) + 'static,
) {
    glib::spawn_future_local(async move {
        if let Ok(value) = gio::spawn_blocking(work).await {
            done(value);
        }
    });
}
