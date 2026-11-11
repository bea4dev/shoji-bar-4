# shoji-bar-4

A Material 3 Expressive desktop shell for [ShojiWM](https://github.com/bea4dev/ShojiWM), written in Rust on [AntiWebUI](https://github.com/bea4dev/AntiWebUI).

![shoji-bar-4 on ShojiWM](./ss.png)

- **Top bar** of floating islands: launcher, workspaces, the active window, a rolling clock, now playing, tray and status icons.
- **Dock** that hides at the bottom edge and rises when the pointer reaches it.
- **Quick settings**, **calendar**, **launcher**, **clipboard history**, **media player**, **notifications**, **OSD** and a **lock screen**.
- **Dynamic color** from a seed or the wallpaper, moved to on a spring everywhere at once, window frames included.
- Springs and shape morphs throughout, and no frames drawn while nothing moves (0% CPU when idle).

## Getting started

You need ShojiWM, a Rust toolchain, and the usual desktop services: NetworkManager, BlueZ, PipeWire, UPower, power-profiles-daemon and logind. `cliphist` and `wl-clipboard` give the clipboard history, and `zenity` lets you pick a user picture.

```sh
git clone https://github.com/bea4dev/shoji-bar-4 ~/.config/shoji-bar-4
cd ~/.config/shoji-bar-4
cargo build --release
```

ShojiWM's default config starts `~/.config/shoji-bar-4/target/release/shoji-bar-4` with the session, binds its keys (`Super` for the launcher, `Super+N` quick settings, `Super+C` calendar, `Super+V` clipboard, `Super+L` lock) and draws the window frames in the shell's colors. With another config, see [Using it with ShojiWM](docs/usage.md#using-it-with-shojiwm).

## This fork

A fork of [bea4dev/shoji-bar-4](https://github.com/bea4dev/shoji-bar-4), mostly written with
[Claude Code](https://claude.com/claude-code) ("vibe coding"), so read it with that in mind. What is different:

- **Core and plugins are separate.** The project is a workspace: `shoji-core` (settings, theme, panel
  manager), `shoji-default` (the stock interface) and `shoji-bar-4` (the binary). Everything that is
  not stock lives in `plugins/`, which is compiled into the binary.
- **Plugins design the UI.** `plugins/init.rs` replaces the whole interface with your own;
  `plugins/init-default.rs` is a **patch mode**: it keeps the stock interface and changes it (islands,
  panels, quick settings, colours) with a `patch(ui: &mut Ui)` function. With an empty plugins
  directory it is the same shell as upstream. See [Customization](docs/customization.md).
- **Example plugins** in `plugins/` (the author's own): a tray drop-down with an agents panel, a
  music panel with the blurred cover, and a palette system (`colour.rs`, `animation.rs`, `menu.rs`,
  `palettes/*.palette`) with a theme page in quick settings, glass palettes and animated
  transitions.
- **`shoji-bar-4 --version`** (`-V`, `version`) prints the version.
- It is meant to be switched with [**alter-bar**](https://github.com/rodrigo-castilla/alter-bar), a small
  tool that changes the Hyprland/Omarchy top bar between `quickshell`, `shoji-bar-4` and any other
  shoji build, and rebuilds a plugin project quickly (`alter-bar reload`).

## Install

You need ShojiWM (or Hyprland with Omarchy, see below), a Rust toolchain, and the usual desktop
services: NetworkManager, BlueZ, PipeWire, UPower, power-profiles-daemon and logind. `cliphist` and
`wl-clipboard` give the clipboard history, and `zenity` lets you pick a user picture.

### With alter-bar (recommended)

On Omarchy/Hyprland, [alter-bar](https://github.com/rodrigo-castilla/alter-bar) (alpha) installs nothing
by itself but does the rest: it builds the project, hides Omarchy's bar without ever killing
quickshell, and starts this one.

```sh
git clone https://github.com/rodrigo-castilla/shoji-bar-4-fork ~/.config/shoji-bar-4-fork
git clone https://github.com/rodrigo-castilla/alter-bar && cargo install --path alter-bar

alter-bar set-bar shoji-bar ~/.config/shoji-bar-4-fork
alter-bar set shoji-bar          # builds if needed, then switches
alter-bar toggle                 # back and forth between bars (bind it to a key)
alter-bar reload                 # after editing plugins/: rebuild and restart in about a second
```

Add `alter-bar apply` to your Hyprland autostart so the chosen bar comes back at login.

### Manual

```sh
git clone https://github.com/rodrigo-castilla/shoji-bar-4-fork ~/.config/shoji-bar-4-fork
cd ~/.config/shoji-bar-4-fork
cargo build --release            # target/release/shoji-bar-4
./target/release/shoji-bar-4 --version
```

Start `target/release/shoji-bar-4` with your session. On ShojiWM, point its config at it (see
[Using it with ShojiWM](docs/usage.md#using-it-with-shojiwm)); under Hyprland, start it after
quickshell when running both. For working on plugins, `cargo build --profile fast` is the quick
build (`target/fast/shoji-bar-4`).

If the plugins in `plugins/` do not build or you only want the stock shell, run
`shoji-bar-4 --default` or empty the `plugins/` directory.

## Documentation

- [Features](docs/features.md): everything the shell does, part by part.
- [Usage](docs/usage.md): running it, ShojiWM integration, commands and settings.
- [Customization](docs/customization.md): changing the interface with Rust code of your own, from a plugins directory.
- [Development](docs/development.md): building against a local AntiWebUI, offscreen previews and CPU reports.
