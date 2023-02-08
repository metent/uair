### v0.5.1

#### Fixed

- Fixed `uair(5)` man page build error.

### v0.5.0

#### Added

- New `uair` format specifier `{state}` and session properties `paused_state_text` and `resumed_state_text`. Allows to display different text depending on the state (paused/resumed) of the timer.
- New `uair` config key: `iterations`. Allows to specify a finite amount of iterations of all sessions.
- New `uairctl` subcommand: `fetch`, to fetch information from the timer and display it in a specified format.
- New `uairctl` subcommand: `finish`, to instantly finish the current session, invoking the session's specified command.
- New `uair` config session property: `time_format`. Specifies the format in which `{time}` format specifier prints time.

#### Changed

- Improve error message by indicating a missing config file. (@thled)

#### Removed

- `-p` and `-r` `uairctl` flags. Use `pause`, `resume` and `toggle` subcommands instead.
- `-h` flag in `uair` and `uairctl`. Use `--help` to display the help message. This is due to a limitation in `argh`, the new argument-parsing library `uair` depends on.

### v0.4.0

- New `uair` config session property: `format` and format specifiers: `{name}`, `{percent}`, `{time}`, `{total}`, `{black}`, `{red}`, `{green}`, `{yellow}`, `{blue}`, `{purple}`, `{cyan}`, `{white}`, `{end}`.
- New `uair` session command environment variables: `$name` and `$duration`.

#### Deprecated

- `after` and `before` session properties in `uair` config. Use `format` property instead.

### v0.3.1

- `uair` performance improvement: prevent allocation of buffer each time a command is received.

### v0.3.0

- Config file and socket file now follow XDG Base Directory Specification.
- New `uair` and `uairctl` command-line flag: -s or --socket. It specifies `uair` server socket path.
- New config file options: `loop_on_end`, `pause_at_start` and `startup_text`.
- Bug Fix: resuming while timer is running should now be a no-op.
- New `uairctl` subcommands: `pause`, `resume` and `toggle`.
- New `uairctl` subcommands: `next` and `prev`, to jump to next and previous sessions.

#### Deprecated

- `-p` and `-r` `uairctl` flags. Use `pause`, `resume` and `toggle` subcommands instead.

### v0.2.0

- Default properties for sessions can now be configured.
- New config file option: autostart. It controls whether a particular session starts automatically.

### v0.1.2

- Command mentioned in the config for a session should now run as intended.

### v0.1.1

- Changed configuration file format from RON to TOML.

### v0.1.0

First public release
