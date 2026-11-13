# Features

## Bar

Floating islands along the top, left or right. **Settings → Bar position** moves the bar immediately.

- **Launcher button**: its Material Shape morphs (Cookie4 → Cookie9 → Circle).
- **Workspaces**: a marker that stretches as it moves, its leading edge on a fast spring and its trailing edge on a slow one. The mouse wheel switches workspaces too. When workspaces come and go, their places open and close and the island follows along the bar.
- **Active window**: on the top bar, its icon and title. A new window, or a new title, comes up from below as the last one goes up, and the island follows its width on a spring, however fast they change. A title wider than the island for the moment fades out at its edge. A few characters changing in one place (a spinner like Claude Code's, a counter, a clock) change where they are, without the title coming up again. A vertical bar shows the application icon only.
- **Clock**: the digits roll as they change, and hovering changes Roboto Flex's weight. A vertical bar stacks hours, minutes and optional seconds, with AM/PM for 12-hour time; the date stays on the top bar.
- **Now playing** (a dancing equalizer): a click opens the player under the top bar or beside a vertical bar, a middle click pauses or plays. The vertical island omits the title.
- **Tray**, network, Bluetooth, volume and battery. Tray menus open beside a vertical bar.

## Dock

The icon under the pointer lifts a little and shows its name.

- It stays hidden below the bottom of the screen. Moving the pointer to the bottom edge brings it up, and it goes down a moment after the pointer leaves. Windows can use the whole height of the screen.
- Pinned apps come first, then a divider, then the other open apps.
- An app that is starting bounces until its window appears.
- Dots underneath show how many windows an app has and which one is focused.
- Right-click menu: new window, desktop actions, pin, close.

## Quick settings

- **Tiles**: round off and change color when on.
- **Sliders** for volume and brightness.
- **Power menu**: Lock, Sleep, Log out, Restart, Shut down.
  - Log out (`shoji_wm --quit`), Restart and Shut down ask first, in a dialog in the middle of the screen over a blurred veil like the launcher's.
  - The dialog's icon shape spins in, and turns into a sun while the pointer is on the action button.
  - Cancel, Escape or a click outside closes it without doing anything.
- **Battery card**:
  - A filling battery picture and rolling numbers.
  - The time until empty or full, and the clock time it will be.
  - A state pill (charging, on battery, full, not charging), power draw, health and charge cycles.
  - Light flows through the battery while it charges.
- **Notifications**: always shown, with "No notifications" when there are none.
- **Settings page**, behind the gear (or the user picture):
  - **User picture**: a copy of the one you pick is kept in `$XDG_STATE_HOME/shoji-bar-4/`. Without one, `~/.face.icon`, `~/.face` or AccountsService's picture is used. Picking one needs zenity.
  - **Style and color**: wallpaper, seed color, scheme variant, and light, auto or dark.
  - **Clock**: 24-hour time and seconds.

## Calendar

- A large clock that rolls down to the seconds.
- The month, from Monday, six weeks of it, with the days of the months around it faint. Today stands on a cookie that turns under the pointer.
  - ‹ and ›, or the mouse wheel, go through the months, which slide over; the month's name goes back to this one.
- Wavy rings for CPU, memory and temperature.

## Launcher

- It opens in the middle of the screen, its items coming in one after another.
- The grid rearranges itself (FLIP) as you search.
- Arrow keys, Enter and Escape work.

## Player

It opens under the top bar's now-playing island, or beside it on a vertical bar.

- A card on a base in the theme's colors, like the other panels. The album art fills the card, darkened so the text stays readable. When the track changes, the art and the title change over.
- Covers can come from local file URLs or HTTP/HTTPS URLs, including redirects. Network covers load in the background (up to 10 MiB, with a 10-second timeout); unavailable covers keep the music-note placeholder. A late download cannot replace a newer track's cover. The last cover is kept, so opening the panel again shows it at once.
- **Wavy seek bar**: a click seeks there, and a drag seeks to where you let go. A handle appears at the end while the pointer is on it.
- Elapsed time and length, and previous, play or pause (a large button that rounds off while playing), and next.

## Clipboard history

What cliphist remembers, newest first, in the middle of the screen.

- **Entries**: text shows an icon (text, link or path) and its first two lines; images show a thumbnail and their size.
- **Search** narrows the list; `png` or `image` finds images.
- **Choosing**: ↑↓ (and PageUp, PageDown) move a highlight that stretches as it goes, and the list springs to keep it in view.
- **Copying**: Enter or a click puts the entry back on the clipboard and closes the list.
- **Forgetting**: Delete (Shift+Delete while searching) or × forgets an entry; its row closes and the ones below move up.
- **Opening**: the rows drop in one after another.
- **Requirement**: cliphist has to be recording, with `wl-paste --watch cliphist store`. ShojiWM's default config starts that.

## Lock screen

It covers every screen with `ext-session-lock`.

- **Look**: a veil over the wallpaper, a large rolling clock and the date, the user picture and name, and the password field. Everything comes in one after another.
- **Typing**: each character adds a Material shape (cookie, clover, flower, ...) that bounces into place. While Enter checks the password, the shapes turn into a loading indicator.
- **Wrong password**: the field shakes and turns red, and says why. When faillock locks the account for a while, PAM's own message is shown.
- **Right password**: everything grows as it fades, the veil lifts, and the desktop with the same wallpaper is back.
- **Keys**: Escape or Ctrl+U clears the field, and Backspace removes one character.
- **Checking**: the password goes to PAM (the `login` service, `pam_authenticate` only) on a thread of its own.
- **When it locks**: on `shoji-bar-4 msg lock`, on logind's Lock (`loginctl lock-session`, or Lock in the quick settings power menu), and before sleep. Nothing outside the lock screen can unlock it.
- **If the shell dies while locked**: ShojiWM stays locked. Start another locker (`swaylock`, ...) in the same session from a TTY; it takes over the lock and can unlock it.
- **faillock**: with Arch's default settings, three wrong passwords in 15 minutes lock the account for 10 minutes, even against the right one.

## Notifications

They work when the shell can become the notification daemon.

- Cards slide in at the top right on a spring.
- The history stays in the quick settings.
- Do not disturb.

## OSD

When the volume or the brightness changes, a pill shows above the dock.

## Theme

- The scheme is made from a seed color with Material You's HCT. The seed can also come from the wallpaper.
- Variants: Tonal, Vibrant, Expressive, Fidelity, Neutral and Mono.
- A new scheme is moved to on a spring, everywhere at once.

## Wallpaper

A new wallpaper crossfades in, settling back from a slight zoom.
