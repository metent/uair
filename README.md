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

Configuration is done in TOML. If a config file is not specified by the `-c` flag, it is sourced according to the XDG Base Directory Specification, i.e. it looks for the config file in the following order, until it successfully finds one.

- $XDG_CONFIG_HOME/uair/uair.toml
- $HOME/.config/uair/uair.toml
- ~/.config/uair/uair.toml

Example Config:

```toml
[defaults]
before = ""
after = "\n"

[[sessions]]
name = "Work"
duration = "30m"
command = "notify-send 'Work Done!'"

[[sessions]]
name = "Rest"
duration = "5m"
command = "notify-send 'Rest Done!'"

[[sessions]]
name = "Work, but harder"
duration = "1h 30m"
command = "notify-send 'Hard Work Done!'"
```

A list of sessions has to be provided in the `sessions` key. Each session is a table with the following keys:

- `name`: name of the session
- `duration`: duration of the session
- `command`: command which is run when the session finishes
- `before`: string which is printed before the remaining time string
- `after`: string which is printed after the remaining time string
- `autostart`: boolean value (true or false) which dictates whether the session automatically starts.

If a property of a session in the array is unspecified, the default value specified in the `defaults` section is used instead. If the property is not mentioned in the default section too, then the propert is sourced from a list of hard-coded defaults.

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
[defaults]
before = ""
after = "\n"

[[sessions]]
name = "Work"
duration = "30m"
command = "notify-send 'Work Done!'"
```

### Simple CLI timer

```toml
[defaults]
before = "\r"
after = "           "

[[sessions]]
name = "Work"
duration = "1h 30m"
command = "notify-send 'Work Done!'"
```

Run with:

```
clear && uair
```

### GUI with yad

```toml
[defaults]
before = "#"
after = "\n"

[[sessions]]
name = "Work"
duration = "1h 30m"
command = "notify-send 'Work Done!'"
```

Run with:

```
uair | yad --progress
```
