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

finish
	Instantly finishes the current session, invoking the session's specified command.

jump [ID]
	Jumps to the session with the given id, [ID].

reload
	Reload the config file. If the new configuration contains a session with the same ID as that of the current session, this session is treated as the new current session, otherwise, the first session is treated as the new current session. The state of the timer (paused or resumed) or the remaining duration of the current session remains unchanged.

fetch [FORMAT]
	Fetches information and displays it in the format specified by the format text [FORMAT]. Formatting of input text is done using the same format specifiers specified in FORMAT SPECIFIERS sections in uair(5).

listen [-o | --override OVERRIDE]
	Output time continuously, while remaining in sync with the main 'uair' instance. Using the optional '-o' flag, a named override specified in uair config can be mentioned, which allows the listening instance to output time in a different format. See 'overrides' property in SESSION PROPERTIES section and the OVERRIDABLES section in uair(5) for more details.

# DESCRIPTION

uairctl is a command line application for controlling uair. It can be binded to a keyboard shortcut for for quickly pausing and resuming the timer.

# SEE ALSO

*uair*(5) *uairctl*(1)
