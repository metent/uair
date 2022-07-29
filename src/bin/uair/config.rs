use std::fs;
use std::io::{self, Write};
use std::time::Duration;
use serde::{Serialize, Deserialize};
use humantime::format_duration;
use super::Args;

pub struct Config {
	pub loop_on_end: bool,
	pub pause_at_start: bool,
	pub startup_text: String,
	pub sessions: Vec<Session>,
}

pub struct Session {
	pub name: String,
	pub duration: Duration,
	pub command: String,
	format: Vec<Token>,
	pub autostart: bool,
}

impl Session {
	pub fn display(&self, time: Duration) -> anyhow::Result<()> {
		let mut stdout = io::stdout();
		for token in &self.format {
			match token {
				Token::Name => write!(stdout, "{}", self.name)?,
				Token::Percent => write!(stdout, "{}", time.as_secs() * 100 / self.duration.as_secs())?,
				Token::Time => write!(stdout, "{}", format_duration(time))?,
				Token::Total => write!(stdout, "{}", format_duration(self.duration))?,
				Token::Color(Color::Black) => write!(stdout, "{}", "\x1b[0;30m")?,
				Token::Color(Color::Red) => write!(stdout, "{}", "\x1b[0;31m")?,
				Token::Color(Color::Green) => write!(stdout, "{}", "\x1b[0;32m")?,
				Token::Color(Color::Yellow) => write!(stdout, "{}", "\x1b[0;33m")?,
				Token::Color(Color::Blue) => write!(stdout, "{}", "\x1b[0;34m")?,
				Token::Color(Color::Purple) => write!(stdout, "{}", "\x1b[0;35m")?,
				Token::Color(Color::Cyan) => write!(stdout, "{}", "\x1b[0;36m")?,
				Token::Color(Color::White) => write!(stdout, "{}", "\x1b[0;37m")?,
				Token::Color(Color::End) => write!(stdout, "{}", "\x1b[0m")?,
				Token::Literal(literal) => write!(stdout, "{}", literal)?,
			};
		}
		stdout.flush()?;
		Ok(())
	}
}

#[derive(Serialize, Deserialize)]
pub struct ConfigBuilder {
	#[serde(default)]
	loop_on_end: bool,
	#[serde(default)]
	pause_at_start: bool,
	#[serde(default)]
	startup_text: String,
	#[serde(default)]
	defaults: Defaults,
	sessions: Vec<SessionBuilder>,
}

impl ConfigBuilder {
	pub fn deserialize(args: &Args) -> anyhow::Result<Self> {
		Ok(toml::from_str(&fs::read_to_string(&args.config)?)?)
	}

	pub fn build(self) -> Config {
		Config {
			loop_on_end: self.loop_on_end,
			pause_at_start: self.pause_at_start,
			startup_text: self.startup_text,
			sessions: self.sessions.into_iter().map(|s| Session {
				name: s.name.unwrap_or_else(|| self.defaults.name.clone()),
				duration: s.duration.unwrap_or_else(|| self.defaults.duration.clone()),
				command: s.command.unwrap_or_else(|| self.defaults.command.clone()),
				format: Self::fetch_format(s.format, s.before, s.after, &self.defaults),
				autostart: s.autostart.unwrap_or_else(|| self.defaults.autostart.clone()),
			}).collect(),
		}
	}

	fn fetch_format(format: Option<String>, before: Option<String>, after: Option<String>, defaults: &Defaults) -> Vec<Token> {
		match format {
			Some(format) => Self::parse(&format),
			None => match (before, after) {
				(Some(before), Some(after)) => Self::from_before_after(before, after),
				(Some(before), None) => Self::from_before_after(before, defaults.after.clone()),
				(None, Some(after)) => Self::from_before_after(defaults.before.clone(), after),
				_ => match &defaults.format {
					Some(format) => Self::parse(format),
					None => Self::from_before_after(defaults.before.clone(), defaults.after.clone())
				}
			}
		}
	}

	fn parse(format: &str) -> Vec<Token> {
		let mut tokens = Vec::new();
		let mut k = 0;
		let mut open = None;

		for (i, c) in format.char_indices() {
			match c {
				'{' => open = Some(i),
				'}' => if let Some(j) = open {
					if let Some(token) = Token::parse(&format[j..=i]) {
						if k != j { tokens.push(Token::Literal(format[k..j].into())) };
						tokens.push(token);
						k = i + 1;
					}
				}
				_ => {},
			}
		}
		if k != format.len() { tokens.push(Token::Literal(format[k..].into())) };

		tokens
	}

	fn from_before_after(before: String, after: String) -> Vec<Token> {
		vec![Token::Literal(before), Token::Time, Token::Literal(after)]
	}
}

#[derive(Serialize, Deserialize)]
pub struct Defaults {
	#[serde(default = "Defaults::name")]
	name: String,
	#[serde(with = "humantime_serde")]
	#[serde(default = "Defaults::duration")]
	duration: Duration,
	#[serde(default = "Defaults::command")]
	command: String,
	#[serde(default = "Defaults::before")]
	before: String,
	#[serde(default = "Defaults::after")]
	after: String,
	#[serde(default = "Defaults::format")]
	format: Option<String>,
	#[serde(default = "Defaults::autostart")]
	autostart: bool,
}

impl Defaults {
	fn name() -> String { "Work".into() }
	fn duration() -> Duration { Duration::from_secs(25 * 60) }
	fn command() -> String { "notify-send 'Session Completed!'".into() }
	fn before() -> String { "".into() }
	fn after() -> String { "\n".into() }
	fn format() -> Option<String> { None }
	fn autostart() -> bool { false }
}

impl Default for Defaults {
	fn default() -> Self {
		Defaults {
			name: Defaults::name(),
			duration: Defaults::duration(),
			command: Defaults::command(),
			before: Defaults::before(),
			after: Defaults::after(),
			format: Defaults::format(),
			autostart: Defaults::autostart(),
		}
	}
}

#[derive(Serialize, Deserialize)]
struct SessionBuilder {
	name: Option<String>,
	#[serde(with = "humantime_serde")]
	#[serde(default)]
	duration: Option<Duration>,
	command: Option<String>,
	before: Option<String>,
	after: Option<String>,
	format: Option<String>,
	autostart: Option<bool>,
}

#[cfg_attr(test, derive(PartialEq, Debug))]
enum Token {
	Name,
	Percent,
	Time,
	Total,
	Color(Color),
	Literal(String),
}

impl Token {
	fn parse(input: &str) -> Option<Self> {
		match input {
			"{name}" => Some(Token::Name),
			"{percent}" => Some(Token::Percent),
			"{time}" => Some(Token::Time),
			"{total}" => Some(Token::Total),
			"{black}" => Some(Token::Color(Color::Black)),
			"{red}" => Some(Token::Color(Color::Red)),
			"{green}" => Some(Token::Color(Color::Green)),
			"{yellow}" => Some(Token::Color(Color::Yellow)),
			"{blue}" => Some(Token::Color(Color::Blue)),
			"{purple}" => Some(Token::Color(Color::Purple)),
			"{cyan}" => Some(Token::Color(Color::Cyan)),
			"{white}" => Some(Token::Color(Color::White)),
			"{end}" => Some(Token::Color(Color::End)),
			_ => None,
		}
	}
}

#[cfg_attr(test, derive(PartialEq, Debug))]
enum Color {
	Black,
	Red,
	Green,
	Yellow,
	Blue,
	Purple,
	Cyan,
	White,
	End,
}

#[cfg(test)]
mod tests {
	use super::{Color, ConfigBuilder, Token};

	#[test]
	fn parse_format() {
		assert_eq!(&ConfigBuilder::parse("{cyan}{time}{end}\n"), &[
			Token::Color(Color::Cyan),
			Token::Time,
			Token::Color(Color::End),
			Token::Literal("\n".into()),
		]);
		assert_eq!(&ConfigBuilder::parse("String with {time} with some text ahead."), &[
			Token::Literal("String with ".into()),
			Token::Time,
			Token::Literal(" with some text ahead.".into())
		]);
		assert_eq!(&ConfigBuilder::parse("}}{}{{}{}}}{{}{{}}}"), &[
			Token::Literal("}}{}{{}{}}}{{}{{}}}".into())
		]);
		assert_eq!(&ConfigBuilder::parse("{time} text {time}"), &[
			Token::Time,
			Token::Literal(" text ".into()),
			Token::Time,
		]);
	}
}
