//! Commands from outside, such as the compositor's key bindings, over a socket in
//! `$XDG_RUNTIME_DIR`:
//!
//! ```text
//! shoji-bar-4 msg toggle launcher [OUTPUT]
//! shoji-bar-4 msg toggle quick-settings [OUTPUT]
//! shoji-bar-4 msg toggle calendar [OUTPUT]
//! shoji-bar-4 msg close
//! shoji-bar-4 msg theme dark|light|auto|toggle
//! shoji-bar-4 msg quit
//! ```
//!
//! Without an output, a panel opens where the window with the keyboard is.

use crate::panels::{self, Panel};
use crate::settings::{ThemeMode, settings};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;

fn socket() -> PathBuf {
    let dir = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let display = std::env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "wayland-0".into());
    dir.join(format!("shoji-bar-4.{display}.sock"))
}

/// Sends a command to the running shell and prints what it says.
pub fn send(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = UnixStream::connect(socket())?;
    writeln!(stream, "{}", args.join(" "))?;
    stream.shutdown(std::net::Shutdown::Write)?;
    let mut answer = String::new();
    BufReader::new(stream).read_line(&mut answer)?;
    let answer = answer.trim();
    if answer != "ok" {
        return Err(answer.to_owned().into());
    }
    Ok(())
}

/// Listens for commands. Call once, from the root.
pub fn serve() {
    let path = socket();
    // Someone else listening means another shell is running; a file with no one behind it is left
    // over from one that died.
    if UnixStream::connect(&path).is_ok() {
        eprintln!("shoji-bar-4: another one is running at {}", path.display());
        std::process::exit(1);
    }
    let _ = std::fs::remove_file(&path);
    let listener = match UnixListener::bind(&path) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("shoji-bar-4: no commands from outside: {e}");
            return;
        }
    };
    let to_ui = ui_channel(
        move |(line, answer): (String, std::sync::mpsc::Sender<String>)| {
            let _ = answer.send(run(&line).map_or_else(|e| e, |()| "ok".into()));
        },
    );
    let _ = std::thread::Builder::new()
        .name("shoji-bar-4 ipc".into())
        .spawn(move || {
            for stream in listener.incoming().flatten() {
                let mut line = String::new();
                if BufReader::new(&stream).read_line(&mut line).is_err() {
                    continue;
                }
                let (tx, rx) = std::sync::mpsc::channel();
                to_ui.send((line.trim().to_owned(), tx));
                let answer = rx
                    .recv_timeout(std::time::Duration::from_secs(2))
                    .unwrap_or_else(|_| "no answer".into());
                let _ = writeln!(&stream, "{answer}");
            }
        });
}

/// The output where the window with the keyboard is, else the first.
fn focused_output() -> Option<String> {
    toplevels()
        .active()
        .and_then(|t| t.outputs.first().cloned())
        .or_else(|| outputs().first().map(|o| o.name.clone()))
}

fn run(line: &str) -> Result<(), String> {
    let words: Vec<&str> = line.split_whitespace().collect();
    match words.as_slice() {
        ["toggle" | "open", name, rest @ ..] => {
            let panel = Panel::from_name(name).ok_or_else(|| format!("no panel called {name}"))?;
            let output = rest
                .first()
                .map(|s| s.to_string())
                .or_else(focused_output)
                .ok_or("no output")?;
            let open_now = panels::current().is_some_and(|(o, p)| o == output && p == panel);
            if words[0] == "toggle" && open_now {
                panels::close();
            } else {
                panels::open(&output, panel);
            }
            Ok(())
        }
        ["close"] => {
            panels::close();
            Ok(())
        }
        ["theme", mode] => {
            let mode = match *mode {
                "dark" => ThemeMode::Dark,
                "light" => ThemeMode::Light,
                "auto" => ThemeMode::Auto,
                "toggle" if crate::theme::is_dark() => ThemeMode::Light,
                "toggle" => ThemeMode::Dark,
                other => return Err(format!("no theme called {other}")),
            };
            settings().update(|s| s.mode = mode);
            Ok(())
        }
        ["quit"] => {
            let _ = std::fs::remove_file(socket());
            quit();
            Ok(())
        }
        _ => Err(format!("unknown command: {line}")),
    }
}
