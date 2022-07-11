uair(5)

# NAME

uair - configuration file

# DESCRIPTION

The number of pomodoro sessions and their properties can be specified by an uair configuration file in TOML format. A config file consists of two keys:

*defaults*
	This is a table containing default session properties. If a session has a property unspecified, the value of the corresponding key in this table is used instead.

*sessions*
	This is an array of tables. Each table in this array corresponds to a session. The order of sessions in this array is the order in which they are scheduled. Each table in this array consists of keys which describe the properties of the session, which are discussed in the following section.

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

*autostart*
	Boolean value (true or false) which dictates whether the session automatically starts.

# SEE ALSO

*uair*(1) *uairctl*(1)
