# uair

`uair` is a minimal pomodoro timer for UNIX-like operating systems. Unlike other timers, `uair` simply prints the remaining time to standard output. Other than making the code more maintainable, this design allows `uair` to be very extensible as it can be used in various status bars and even command-line and graphical user interfaces.

## Features

- Extensible: Can be used in status bars, desktop widgets, CLIs, GUIs, etc.
- Keyboard-driven: Uses `uairctl` command-line utility for pausing/resuming the timer. It can be binded to a keyboard shortcut.
- Resource-efficient: Uses concurrency instead of spawing multiple threads.

## Usage

Configuration is done in RON. Default config path is `$HOME/.config/uair/uair.ron`.

Example Config:

```
(
	sessions: [
		(
			name: "Work",
			duration: "30m",
			command: "notify-send 'Work Done!'",
			before: "",
			after: "\n",
		),
		(
			name: "Rest",
			duration: "5m",
			command: "notify-send 'Rest Done!'",
			before: "",
			after: "\n",
		),
		(
			name: "Work, but harder",
			duration: "1h",
			command: "notify-send 'Hard Work Done!'",
			before: "",
			after: "\n",
		),
	]
)
```

A list of sessions has to be provided in the top-level `sessions` key. Each session is a struct with the following keys:

- `name`: name of the session
- `duration`: duration of the session
- `command`: command which is run when the session finishes
- `before`: string which is printed before the remaining time string
- `after`: string which is printed after the remaining time string

The timer can be paused or resumed using the `uairctl` command. While the timer is running, pause using

```
uairctl -p
```

and resume using

```
uairctl -r
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
exec = /home/rishabh/bin/uair
label = %output%
tail = true
```

Remember to include the module in the modules list.

```
modules-right = filesystem uair pulseaudio xkeyboard memory cpu wlan eth date
```

In order for it to be displayed, a newline should be printed after printing the remaining time.

```
(
	sessions: [
		(
			name: "Work",
			duration: "30m",
			command: "notify-send 'Work Done!'",
			before: "",
			after: "\n",
		),
	]
)
```

### Simple CLI timer

```
(
	sessions: [
		(
			name: "Work",
			duration: "1h 30m",
			command: "notify-send 'Work Done!'",
			before: "\r",
			after: "           ",
		),
	]
)
```

Run with:

`clear && uair`

### GUI with yad

```
(
	sessions: [
		(
			name: "Work",
			duration: "1h 30m",
			command: "notify-send 'Work Done!'",
			before: "#",
			after: "\n",
		),
	]
)
```

Run with:

`uair | yad --progress`
