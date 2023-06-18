# uair

`uair` is a minimal pomodoro timer for UNIX-like operating systems. Unlike other timers, `uair` simply prints the remaining time to standard output. Other than making the code more maintainable, this design allows `uair` to be very extensible as it can be used in various status bars and even command-line and graphical user interfaces.

## Features

- Extensible: Can be used in status bars, desktop widgets, CLIs, GUIs, etc.
- Keyboard-driven: Uses `uairctl` command-line utility for pausing/resuming the timer. It can be binded to a keyboard shortcut.
- Resource-efficient: Uses concurrency instead of spawing multiple threads.
- Multiple synchronized timers: Multiple synchronized `uair` timers can co-exist while sharing a single handle
- Minimal: It adheres to the UNIX philosophy of focusing in doing one thing, and doing it well.

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

### Quickstart

Copy `resources/uair.toml` under the project directory to `~/.config/uair/`.

```
mkdir -p ~/.config/uair
cp -r resources/uair.toml ~/.config/uair/uair.toml
```

Start uair.

```
uair
```

When `uair` is started, or a session is completed, the timer is in a paused state. In order to start the session, `uairctl` command must be used. Start the session by resuming the timer by invoking `uairctl` from another shell.

```
uairctl resume
```

and pause the session using

```
uairctl pause
```

To toggle between pause and resume states, use

```
uairctl toggle
```

To start another instance synced with this one, you can use
```
uairctl listen
```

### Configuration

Configuration is done in TOML. If a config file is not specified by the `-c` flag, it is sourced according to the XDG Base Directory Specification, i.e. it looks for the config file in the following order, until it successfully finds one.

- $XDG_CONFIG_HOME/uair/uair.toml
- $HOME/.config/uair/uair.toml
- ~/.config/uair/uair.toml

Example Config:

```toml
[defaults]
format = "{time}\n"

[[sessions]]
id = "work"
name = "Work"
duration = "30m"
command = "notify-send 'Work Done!'"

[[sessions]]
id = "rest"
name = "Rest"
duration = "5m"
command = "notify-send 'Rest Done!'"

[[sessions]]
id = "hardwork"
name = "Work, but harder"
duration = "1h 30m"
command = "notify-send 'Hard Work Done!'"
```

A list of sessions has to be provided in the `sessions` key. Each session is a table containing the properties of the session. Some of those properties are listed as follows:

- `id`: unique identifier of the session
- `name`: name of the session
- `duration`: duration of the session
- `command`: command which is run when the session finishes
- `format`: specifies the format in which text is printed each second
- `autostart`: boolean value (true or false) which dictates whether the session automatically starts.

If a property of a session in the array is unspecified, the default value specified in the `defaults` section is used instead. The exception to this rule is the `id` property which, if unspecified, defaults to the index(starting from 0) of session in the sessions array. If the property is not mentioned in the default section too, then the property is sourced from a list of hard-coded defaults.

It is recommended to specify an `id` for every session as it makes it possible for `uair` to keep track of the current session while reloading the config file. It also makes it convenient to jump to any session using its `id` using `uairctl jump` command.

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
format = "{time}\n"

[[sessions]]
id = "work"
name = "Work"
duration = "30m"
command = "notify-send 'Work Done!'"
```

### Simple CLI timer

```toml
[defaults]
format = "\r{time}           "

[[sessions]]
id = "work"
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
format = "{percent}\n#{time}\n"

[[sessions]]
id = "work"
name = "Work"
duration = "1h 30m"
command = "notify-send 'Work Done!'"
```

Run with:

```
uair | yad --progress --no-buttons --css="* { font-size: 80px; }"
```

## Roadmap

- [X] Basic pause/resume functionality using UNIX sockets
- [X] next/prev subcommands
- [X] Format specifiers
- [X] Ability to reload configuration
- [X] uairctl listen subcommand: for multiple synchonized timers
- [ ] Dedicated GUI client
- [ ] Dedicated crossterm client
