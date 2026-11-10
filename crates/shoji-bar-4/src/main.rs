//! shoji-bar-4: a Material 3 Expressive desktop shell for ShojiWM.
//!
//! The interface comes from the plugins directory when it has an `init.rs` or an `init-default.rs`
//! (see `build.rs`), otherwise it is the stock one. Starting with `--default` or with
//! `SHOJI_SAFE=1` in the environment ignores the plugins and runs the stock interface, which is
//! the way out when code of your own does not start.

mod plugin {
    include!(concat!(env!("OUT_DIR"), "/plugin_entry.rs"));
}

fn main() {
    let safe = std::env::var_os("SHOJI_SAFE").is_some_and(|v| !v.is_empty() && v != "0")
        || std::env::args().nth(1).as_deref() == Some("--default");
    if safe {
        shoji_default::run();
    } else {
        if !plugin::SOURCE.is_empty() && std::env::args().nth(1).is_none() {
            eprintln!("shoji-bar-4: interface from {}", plugin::SOURCE);
        }
        plugin::run();
    }
}
