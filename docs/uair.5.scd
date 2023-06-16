uair(5)

# NAME

uair - configuration file

# DESCRIPTION

The number of pomodoro sessions and their properties can be specified by an uair configuration file in TOML format. A config file consists of the following keys:

*loop_on_end*
	This is a boolean value (true or false) which controls whether uair repeats all sessions after the completion of the last session.

*iterations*
	This is a non-negative integer which specifies the number of times sessions mentioned in the sessions array are iterated.

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
	Command which is run when the session finishes. See COMMAND ENVIRONMENT section for information on environment variables which are passed to the command.

*before*
	String which is printed before the remaining time string. (Deprecated)

*after*
	String which is printed after the remaining time string. (Deprecated)

*format*
	Specifies the format in which text is printed each second. See FORMAT SPECIFIERS section for details.

*time_format*
	Specifies the format in which *{time}* format specifier prints time. See TIME FORMAT SPECIFIERS section for details.

*autostart*
	Boolean value (true or false) which dictates whether the session automatically starts.

*paused_state_text*
	Text which is displayed by the *{state}* format specifier when the timer is paused.

*resumed_state_text*
	Text which is displayed by the *{state}* format specifier when the timer is resumed.

*overrides*
	A table consisting of OVERRIDABLES as values and their names as keys. It allows to specify named overrides to be applied during `uairctl listen` See OVERRIDABLES section for more information and for the list of overridable properties.

# FORMAT SPECIFIERS

The format property of a session is a string which specifies what and how text is printed every second. For example, the following format string can be used to print the name of the session followed by the remaining time, followed by the total duration of the session, all in cyan color.

	{cyan}{name}: {time} / {total}{end}

The list of format specifiers is:

*{name}*
	Session name

*{percent}*
	Percentage of time remaining

*{time}*
	Remaining time for session

*{state}*
	Text which depends on the state (paused/resumed) of the timer. This text is configurable through *paused_state_text* and *resumed_state_text* session properties.

*{total}*
	Total duration of session

*{black}*
	Start black color text

*{red}*
	Start red color text

*{green}*
	Start green color text

*{yellow}*
	Start yellow color text

*{blue}*
	Start blue color text

*{purple}*
	Start purple color text

*{cyan}*
	Start cyan color text

*{white}*
	Start white color text

*{end}*
	Start end color text

# TIME FORMAT SPECIFIERS

The time_format property of a session is a string which specifies what and how text produced by *{time}* format specifier is printed. Time format specifiers have the following syntax.

	%[optional skip flag][optional padding flag]alphabet

As an example, the following time format string can be used to print time in a digital clock-like format.

	%H:%M:%S

The list of interpreted sequences is:

*%Y*
	Remaining years

*%B*
	Remaining months in a year

*%D*
	Remaining days in a month

*%H*
	Remaining hours in a day

*%M*
	Remaining minutes in an hour

*%S*
	Remaining seconds in a minute

*%P*
	Prints 's' if the quantity specified by the format specifier before it is plural and nothing if singular.

The list of optional padding flags is:

*0*
	Pad with zeroes (default)

*\_*
	Pad with spaces

*-*
	Do not pad

An optional '\*' flag may follow '%' to skip the quantity specified by the format specifier and skip all text before the next quantifiable format specifier.

# OVERRIDABLES

Each value in the 'overrides' session property is a table named OVERRIDABLES which can contain one or more of the following properties.

*format*
	Specifies the format in which text is printed each second. See FORMAT SPECIFIERS section for details.

*time_format*
	Specifies the format in which *{time}* format specifier prints time. See TIME FORMAT SPECIFIERS section for details.

*paused_state_text*
	Text which is displayed by the *{state}* format specifier when the timer is paused.

*resumed_state_text*
	Text which is displayed by the *{state}* format specifier when the timer is resumed.

# COMMAND ENVIRONMENT

Some environment variables are passed to the command specified by the command property of a session which enables printing various session properties. They are as follows

*$name*
	name property of session

*$duration*
	duration property of session

# SEE ALSO

*uair*(1) *uairctl*(1)
