//! Being the notification daemon, when no one else is.

use anti_webui::shell::prelude::*;
use std::cell::OnceCell;

thread_local! {
    static SERVER: OnceCell<Option<NotificationServer>> = const { OnceCell::new() };
}

/// The notification server, if this got the name on the bus.
pub fn server() -> Option<NotificationServer> {
    SERVER.with(|s| {
        s.get_or_init(|| match notification_server() {
            Ok(server) => Some(server),
            Err(e) => {
                eprintln!("shoji-bar-4: not showing notifications: {e}");
                None
            }
        })
        .clone()
    })
}
