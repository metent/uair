# uair

`uair` is a minimal pomodoro timer for UNIX-like operating systems. Unlike other timers, `uair` simply prints the remaining time to standard output. Other than making the code more maintainable, this design allows `uair` to be very extensible as it can be used in various status bars and even command-line and graphical user interfaces.

## Features

- Extensible: Can be used in status bars, desktop widgets, CLIs, GUIs, etc.
- Keyboard-driven: Uses `uairctl` command-line utility for pausing/resuming the timer. It can be binded to a keyboard shortcut.
- Resource-efficient: Uses concurrency instead of spawing multiple threads.

## Installation

### From Arch User Repository

`uair` is [packaged](https://aur.archlinux.org/packages/uair) for the AUR. Use an AUR helper or `makepkg` to install.

```
paru -S uair
```

or

```
yay -S uair
```

or

```
git clone https://aur.archlinux.org/uair.git
cd uair
makepkg -si
```

### From crates.io

```
cargo install uair
```

Make sure to include `$HOME/.cargo/bin` in the `PATH` variable.

## Usage

Configuration is done in TOML. Default config path is `$HOME/.config/uair/uair.toml`.

Example Config:

```toml
[[sessions]]
name = "Work"
duration = "30m"
command = "notify-send 'Work Done!'"
before = ""
after = "\n"

[[sessions]]
name = "Rest"
duration = "5m"
command = "notify-send 'Rest Done!'"
before = ""
after = "\n"

[[sessions]]
name = "Work, but harder"
duration = "1h 30m"
command = "notify-send 'Hard Work Done!'"
before = ""
after = "\n"
```

A list of sessions has to be provided in the top-level `sessions` key. Each session is a struct with the following keys:

- `name`: name of the session
- `duration`: duration of the session
- `command`: command which is run when the session finishes
- `before`: string which is printed before the remaining time string
- `after`: string which is printed after the remaining time string

When `uair` is started, or a session is completed, the timer is in a paused state. In order to start the session, `uairctl` command must be used. Start the session by resuming the timer.

```
uairctl -r
```

While the timer is running, pause using

```
uairctl -p
```

To toggle between pause and resume states, use both flags

```
uairctl -p -r
```

### Integration with polybar

Include pomodoro module in the config.

```ini
[module/uair]
type = custom/script
exec = uair
label = %output%
tail = true
```

Remember to include the module in the modules list.

```
modules-right = filesystem uair pulseaudio xkeyboard memory cpu wlan eth date
```

In order for it to be displayed, a newline should be printed after printing the remaining time.

```toml
[[sessions]]
name = "Work"
duration = "30m"
command = "notify-send 'Work Done!'"
before = ""
after = "\n"
```

### Simple CLI timer

```toml
[[sessions]]
name = "Work"
duration = "1h 30m"
command = "notify-send 'Work Done!'"
before = "\r"
after = "           "
```

Run with:

```
clear && uair
```

### GUI with yad

```toml
[[sessions]]
name = "Work"
duration = "1h 30m"
command = "notify-send 'Work Done!'"
before = "#"
after = "\n"
```

Run with:

```
uair | yad --progress
```
