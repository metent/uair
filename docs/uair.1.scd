uair(1)

# NAME

uair - An extensible pomodoro timer

# SYPNOSIS

*uair* [options..]

# OPTIONS

*-c, --config*
	Specifies a config file.

*-h, --help*
	Show help message and quit.

# DESCRIPTION

uair is a minimal pomodoro timer for UNIX-like operating systems. Unlike other timers, uair simply prints the remaining time to standard output. Other than making the code more maintainable, this design allows uair to be very extensible as it can be used in various status bars and even command-line and graphical user interfaces.

# CONFIGURATION

Configuration is done in TOML. Default config path is $HOME/.config/uair/uair.toml. For information on the config file format, see uair(5).

# SEE ALSO

*uair*(5) *uairctl*(1)
