uair(5)

# NAME

uair - configuration file

# DESCRIPTION

The number of pomodoro sessions and their properties can be specified by an uair configuration file in TOML format. A config file consists of the following keys:

*loop_on_end*
	This is a boolean value (true or false) which controls whether uair repeats all sessions after the completion of the last session.

*pause_at_start*
	This is a boolean value (true or false) which controls whether uair is at paused state at startup.

*startup_text*
	It specifies the text to be printed at startup.

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
	String which is printed before the remaining time string. (Deprecated)

*after*
	String which is printed after the remaining time string. (Deprecated)

*format*
	Specifies the format in which text is printed each second. See FORMAT SPECIFIERS section for details.

*autostart*
	Boolean value (true or false) which dictates whether the session automatically starts.

# FORMAT SPECIFIERS

The format property of a session is a string which specifies what and how text is printed every second. For example, the following format string can be used to print the name of the session followed by the remaining time, followed by the total duration of the session, all in cyan color.

	{cyan}{name}: {time} / {total}{end}

The list of format specifiers is:

{name}
	Session name

{percent}
	Percentage of time remaining

{time}
	Remaining time for session

{total}
	Total duration of session

{black}
	Start black color text

{red}
	Start red color text

{green}
	Start green color text

{yellow}
	Start yellow color text

{blue}
	Start blue color text

{purple}
	Start purple color text

{cyan}
	Start cyan color text

{white}
	Start white color text

{end}
	Start end color text

# SEE ALSO

*uair*(1) *uairctl*(1)
