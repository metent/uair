### Unreleased

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
