uair(5)

# NAME

uair - configuration file

# DESCRIPTION

The number of pomodoro sessions and their properties can be specified by an uair configuration file in TOML format. The top-level struct contains a 'sessions' key, to which a list of sessions must be assigned. Each session is a object which contains keys listed in the following section.

# SESSION PROPERTIES

*name*
	Name of the session.

*duration*
	Duration of the session. Can be specified in human readable format. e.g.: "1h 47m"

*command*
	Command which is run when the session finishes.

*before*
	String which is printed before the remaining time string.

*after*
	String which is printed after the remaining time string.

# SEE ALSO

*uair*(1) *uairctl*(1)
