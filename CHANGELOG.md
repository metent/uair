### Unreleased

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
