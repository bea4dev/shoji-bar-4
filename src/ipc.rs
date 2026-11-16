//! Commands from outside, such as the compositor's key bindings, over a socket in
//! `$XDG_RUNTIME_DIR`:
//!
//! ```text
//! shoji-bar-4 msg toggle launcher [OUTPUT]
//! shoji-bar-4 msg toggle quick-settings [OUTPUT]
//! shoji-bar-4 msg toggle calendar [OUTPUT]
//! shoji-bar-4 msg toggle clipboard [OUTPUT]
//! shoji-bar-4 msg toggle media [OUTPUT]
//! shoji-bar-4 msg close
//! shoji-bar-4 msg lock
//! shoji-bar-4 msg theme dark|light|auto|toggle
//! shoji-bar-4 msg padding [status|reset|<N>|gaps <N>|<top|right|bottom|left> <N|auto>|[set] <k=v>...]
//! shoji-bar-4 msg quit
//! shoji-bar-4 msg status
//! ```
//!
//! Without an output, a panel opens where the window with the keyboard is.

use crate::panels::{self, Panel};
use crate::settings::{ShellPadding, ThemeMode, settings};
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
    match answer.strip_prefix("ok") {
        Some(said) if said.is_empty() || said.starts_with(' ') => {
            if !said.is_empty() {
                println!("{}", said.trim_start());
            }
            Ok(())
        }
        _ => Err(answer.to_owned().into()),
    }
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
            let _ = answer.send(match run(&line) {
                Ok(said) if said.is_empty() => "ok".into(),
                Ok(said) => format!("ok {said}"),
                Err(e) => e,
            });
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

/// What could keep the shell busy, on one line, for `scripts/cpu-report.sh`.
fn status() -> String {
    let panel = panels::current().map_or("none".into(), |(output, panel)| {
        format!("{panel:?}@{output}")
    });
    let playing = media()
        .current()
        .map_or("none".into(), |p| format!("{:?}", p.status));
    let notifications = crate::notify::server().map_or("off".into(), |s| {
        format!("{}/{}", s.popups().len(), s.notifications().len())
    });
    format!(
        "panel={panel} ticking={} media={playing} popups/notifications={notifications} \
         windows={} workspaces={} tray={} dark={}",
        crate::ui::ticking(),
        toplevels().all().len(),
        workspaces().all().len(),
        tray().items().len(),
        crate::theme::is_dark(),
    )
}

/// The output where the window with the keyboard is, else the first.
pub fn focused_output() -> Option<String> {
    toplevels()
        .active()
        .and_then(|t| t.outputs.first().cloned())
        .or_else(|| outputs().first().map(|o| o.name.clone()))
}

fn run(line: &str) -> Result<String, String> {
    run_with(line, focused_output)
}

fn run_with(
    line: &str,
    resolve_output: impl FnOnce() -> Option<String>,
) -> Result<String, String> {
    let words: Vec<&str> = line.split_whitespace().collect();
    match words.as_slice() {
        ["toggle" | "open", name, rest @ ..] => {
            let panel = Panel::from_name(name).ok_or_else(|| format!("no panel called {name}"))?;
            if !panel.enabled() {
                return Err(format!("{name} is disabled in Settings"));
            }
            let output = rest
                .first()
                .map(|s| s.to_string())
                .or_else(resolve_output)
                .ok_or("no output")?;
            let open_now = panels::current().is_some_and(|(o, p)| o == output && p == panel);
            if words[0] == "toggle" && open_now {
                panels::close();
            } else {
                panels::open(&output, panel);
            }
            Ok(String::new())
        }
        // There is no unlock: only the password does that.
        ["lock"] => {
            crate::lock::lock();
            Ok(String::new())
        }
        ["close"] => {
            panels::close();
            Ok(String::new())
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
            Ok(String::new())
        }
        ["padding", rest @ ..] => run_padding(rest),
        ["status"] => Ok(status()),
        ["quit"] => {
            let _ = std::fs::remove_file(socket());
            quit();
            Ok(String::new())
        }
        _ => Err(format!("unknown command: {line}")),
    }
}

fn padding_status() -> String {
    let p = settings().with(|s| s.padding.normalized());
    let edge = |v: Option<u16>| match v {
        Some(n) => n.to_string(),
        None => format!("auto({})", p.gaps),
    };
    format!(
        "gaps={} top={} right={} bottom={} left={}",
        p.gaps,
        edge(p.top),
        edge(p.right),
        edge(p.bottom),
        edge(p.left),
    )
}

fn parse_gaps_value(raw: &str) -> Result<u16, String> {
    raw.parse::<u16>()
        .ok()
        .filter(|&n| n <= ShellPadding::MAX_GAPS)
        .ok_or_else(|| format!("gaps must be 0..={}", ShellPadding::MAX_GAPS))
}

fn parse_edge_value(raw: &str, side: &str) -> Result<Option<u16>, String> {
    if raw.eq_ignore_ascii_case("auto") {
        return Ok(None);
    }
    raw.parse::<u16>()
        .ok()
        .filter(|&n| n <= ShellPadding::MAX_EDGE)
        .map(Some)
        .ok_or_else(|| format!("{side} padding must be 0..={} or auto", ShellPadding::MAX_EDGE))
}

fn apply_padding_kv(padding: &mut ShellPadding, token: &str) -> Result<(), String> {
    let (key, value) = token
        .split_once('=')
        .ok_or_else(|| format!("expected key=value, got {token}"))?;
    match key {
        "gaps" => padding.gaps = parse_gaps_value(value)?,
        "top" => padding.top = parse_edge_value(value, "top")?,
        "right" => padding.right = parse_edge_value(value, "right")?,
        "bottom" => padding.bottom = parse_edge_value(value, "bottom")?,
        "left" => padding.left = parse_edge_value(value, "left")?,
        other => return Err(format!("unknown padding key: {other}")),
    }
    Ok(())
}

fn run_padding(words: &[&str]) -> Result<String, String> {
    match words {
        [] | ["status"] => Ok(padding_status()),
        ["reset"] => {
            settings().update(|s| s.padding = ShellPadding::DEFAULT);
            Ok(String::new())
        }
        ["gaps", value] => {
            let gaps = parse_gaps_value(value)?;
            settings().update(|s| {
                let mut next = s.padding.normalized();
                next.gaps = gaps;
                s.padding = next;
            });
            Ok(String::new())
        }
        [side @ ("top" | "right" | "bottom" | "left"), value] => {
            let edge = parse_edge_value(value, side)?;
            settings().update(|s| {
                let mut next = s.padding.normalized();
                match *side {
                    "top" => next.top = edge,
                    "right" => next.right = edge,
                    "bottom" => next.bottom = edge,
                    "left" => next.left = edge,
                    _ => unreachable!(),
                }
                s.padding = next;
            });
            Ok(String::new())
        }
        ["set"] => Err("padding set requires at least one key=value".into()),
        ["set", assignments @ ..] => {
            let mut next = settings().with(|s| s.padding.normalized());
            for token in assignments {
                apply_padding_kv(&mut next, token)?;
            }
            settings().update(|s| s.padding = next);
            Ok(String::new())
        }
        [single] if !single.contains('=') => {
            if single.parse::<i64>().is_ok() {
                let gaps = parse_gaps_value(single)?;
                settings().update(|s| {
                    s.padding = ShellPadding {
                        gaps,
                        ..ShellPadding::DEFAULT
                    };
                });
                Ok(String::new())
            } else {
                Err(format!("unknown padding command: {single}"))
            }
        }
        assignments if !assignments.is_empty() => {
            let mut next = settings().with(|s| s.padding.normalized());
            for token in assignments {
                apply_padding_kv(&mut next, token)?;
            }
            settings().update(|s| s.padding = next);
            Ok(String::new())
        }
        _ => Err("unknown padding command".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_launcher_is_rejected_while_other_panels_open() {
        crate::settings::never_save();
        settings().set(crate::settings::Settings::default());
        panels::close();

        settings().update(|s| s.launcher_enabled = false);
        assert_eq!(
            run_with("toggle launcher DP-1", || Some("DP-1".into())),
            Err("launcher is disabled in Settings".into())
        );
        assert_eq!(
            run_with("open launcher", || Some("DP-1".into())),
            Err("launcher is disabled in Settings".into())
        );
        assert_eq!(panels::current(), None);

        run_with("open calendar", || Some("DP-1".into())).unwrap();
        assert_eq!(panels::current(), Some(("DP-1".into(), Panel::Calendar)));

        panels::close();
        settings().set(crate::settings::Settings::default());
    }

    #[test]
    fn padding_ipc_updates_queries_resets_and_validates_atomically() {
        crate::settings::never_save();
        settings().set(crate::settings::Settings::default());

        assert_eq!(
            run_with("padding", || None).unwrap(),
            "gaps=14 top=auto(14) right=auto(14) bottom=auto(14) left=auto(14)"
        );

        run_with("padding top 8", || None).unwrap();
        run_with("padding gaps 20", || None).unwrap();
        assert_eq!(
            settings().with(|s| s.padding),
            ShellPadding {
                gaps: 20,
                top: Some(8),
                right: None,
                bottom: None,
                left: None,
            }
        );

        run_with("padding set left=64 right=64 bottom=0", || None).unwrap();
        assert_eq!(
            run_with("padding status", || None).unwrap(),
            "gaps=20 top=8 right=64 bottom=0 left=64"
        );

        run_with("padding top auto", || None).unwrap();
        assert_eq!(settings().with(|s| s.padding.top), None);

        // `padding <N>` sets uniform padding and clears per-edge overrides.
        run_with("padding 16", || None).unwrap();
        assert_eq!(
            settings().with(|s| s.padding),
            ShellPadding {
                gaps: 16,
                ..ShellPadding::DEFAULT
            }
        );

        // Invalid assignments must not mutate existing state.
        assert!(run_with("padding set", || None).is_err());
        assert!(run_with("padding 65", || None).is_err());
        assert!(run_with("padding -1", || None).is_err());
        assert!(run_with("padding top 513", || None).is_err());
        assert!(run_with("padding set gaps=8 left=999", || None).is_err());
        assert_eq!(
            settings().with(|s| s.padding),
            ShellPadding {
                gaps: 16,
                ..ShellPadding::DEFAULT
            }
        );

        run_with("padding gaps=12 left=48", || None).unwrap();
        assert_eq!(
            settings().with(|s| s.padding),
            ShellPadding {
                gaps: 12,
                top: None,
                right: None,
                bottom: None,
                left: Some(48),
            }
        );

        run_with("padding reset", || None).unwrap();
        assert_eq!(settings().with(|s| s.padding), ShellPadding::DEFAULT);
        settings().set(crate::settings::Settings::default());
    }
}
