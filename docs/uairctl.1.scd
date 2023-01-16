uairctl(1)

# NAME

uairctl - Command-line application for controlling uair

# SYPNOSIS

*uairctl* [options..] command

# OPTIONS

*-s, --socket*
	Specifies a socket file.

*--help*
	Show help message and quit.

# COMMANDS

pause
	Pauses the timer.

resume
	Resumes the timer.

toggle
	Toggles the state of the timer.

next
	Jumps to the next session.

prev
	Jumps to the previous session.

fetch [FORMAT]
	Fetches information and displays it in the format specified by the format text [FORMAT]. Formatting of input text is done using the same format specifiers specified in FORMAT SPECIFIERS sections in uair(5).

# DESCRIPTION

uairctl is a command line application for controlling uair. It can be binded to a keyboard shortcut for for quickly pausing and resuming the timer.

# SEE ALSO

*uair*(5) *uairctl*(1)
