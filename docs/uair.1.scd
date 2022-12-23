uair(1)

# NAME

uair - An extensible pomodoro timer

# SYPNOSIS

*uair* [options..]

# OPTIONS

*-c, --config*
	Specifies a config file.

*-s, --socket*
	Specifies a socket file.

*--help*
	Show help message and quit.

# DESCRIPTION

uair is a minimal pomodoro timer for UNIX-like operating systems. Unlike other timers, uair simply prints the remaining time to standard output. Other than making the code more maintainable, this design allows uair to be very extensible as it can be used in various status bars and even command-line and graphical user interfaces.

# CONFIGURATION

Configuration is done in TOML. If a config file is not specified by the *-c* flag, it is sourced according to the XDG Base Directory Specification, i.e. it looks for the config file in the following order, until it successfully finds one.

- $XDG_CONFIG_HOME/uair/uair.toml
- $HOME/.config/uair/uair.toml
- ~/.config/uair/uair.toml

For information on the config file format, see uair(5).

# SEE ALSO

*uair*(5) *uairctl*(1)
