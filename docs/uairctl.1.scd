uairctl(1)

# NAME

uairctl - Command-line application for controlling uair

# SYPNOSIS

*uairctl* [options..] command

# OPTIONS

*-p, --pause*
	Pauses the timer. (Deprecated)

*-r, --resume*
	Resumes the timer. (Deprecated)

*-s, --socket*
	Specifies a socket file.

*-h, --help*
	Show help message and quit.

# COMMANDS

pause
	Pauses the timer.

resume
	Resumes the timer.

toggle
	Toggles the state of the timer.

# DESCRIPTION

uairctl is a command line application for controlling uair. It can be binded to a keyboard shortcut for for quickly pausing and resuming the timer. *-p* and *-r* flags can be used together for toggling between 'paused' and 'resumed' states. Note however, both these flags are deprecated and may be removed in subsequent releases.

	uairctl -p -r

# SEE ALSO

*uair*(5) *uairctl*(1)
