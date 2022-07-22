use std::fs;
use std::io::{self, Write};
use std::time::Duration;
use serde::{Serialize, Deserialize};
use humantime::FormattedDuration;
use super::Args;

pub struct Config {
	pub loop_on_end: bool,
	pub pause_at_start: bool,
	pub startup_text: String,
	pub sessions: Vec<SessionConfig>,
}

pub struct SessionConfig {
	pub name: String,
	pub duration: Duration,
	pub command: String,
	pub before: String,
	pub after: String,
	pub format: OutputFormat,
	pub autostart: bool,
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
	sessions: Vec<SessionConfigBuilder>,
}

impl ConfigBuilder {
	pub fn parse(args: &Args) -> anyhow::Result<Self> {
		Ok(toml::from_str(&fs::read_to_string(&args.config)?)?)
	}

	pub fn build(self) -> Config {
		Config {
			loop_on_end: self.loop_on_end,
			pause_at_start: self.pause_at_start,
			startup_text: self.startup_text,
			sessions: self.sessions.into_iter().map(|s| SessionConfig {
				name: s.name.unwrap_or_else(|| self.defaults.name.clone()),
				duration: s.duration.unwrap_or_else(|| self.defaults.duration.clone()),
				command: s.command.unwrap_or_else(|| self.defaults.command.clone()),
				before: s.before.unwrap_or_else(|| self.defaults.before.clone()),
				after: s.after.unwrap_or_else(|| self.defaults.after.clone()),
				format: OutputFormat::parse(&s.format.unwrap_or_else(|| self.defaults.format.clone())),
				autostart: s.autostart.unwrap_or_else(|| self.defaults.autostart.clone()),
			}).collect(),
		}
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
	format: String,
	#[serde(default = "Defaults::autostart")]
	autostart: bool,
}

impl Defaults {
	fn name() -> String { "Work".into() }
	fn duration() -> Duration { Duration::from_secs(25 * 60) }
	fn command() -> String { "notify-send 'Session Completed!'".into() }
	fn before() -> String { "".into() }
	fn after() -> String { "\n".into() }
	fn format() -> String { "{time}".into() }
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
struct SessionConfigBuilder {
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
pub struct OutputFormat {
	tokens: Vec<Token>,
}

impl OutputFormat {
	fn parse(format: &str) -> Self {
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

		OutputFormat { tokens }
	}

	pub fn display(&self, time: FormattedDuration, before: &str, after: &str) -> anyhow::Result<()> {
		let mut stdout = io::stdout();
		write!(stdout, "{}", before)?;
		for token in &self.tokens {
			match token {
				Token::Time => write!(stdout, "{}", time)?,
				Token::Color(Color::Black) => write!(stdout, "{}", "\x1b[0;30m")?,
				Token::Color(Color::Red) => write!(stdout, "{}", "\x1b[0;31m")?,
				Token::Color(Color::Green) => write!(stdout, "{}", "\x1b[0;32m")?,
				Token::Color(Color::Yellow) => write!(stdout, "{}", "\x1b[0;33m")?,
				Token::Color(Color::Blue) => write!(stdout, "{}", "\x1b[0;34m")?,
				Token::Color(Color::Purple) => write!(stdout, "{}", "\x1b[0;35m")?,
				Token::Color(Color::Cyan) => write!(stdout, "{}", "\x1b[0;36m")?,
				Token::Color(Color::White) => write!(stdout, "{}", "\x1b[0;37m")?,
				Token::Color(Color::End) => write!(stdout, "{}", "\x1b[0m")?,
				_ => {},
			};
		}
		write!(stdout, "{}", after)?;
		stdout.flush()?;
		Ok(())
	}
}

#[cfg_attr(test, derive(PartialEq, Debug))]
enum Token {
	Time,
	Color(Color),
	Literal(String),
}

impl Token {
	fn parse(input: &str) -> Option<Self> {
		match input {
			"{time}" => Some(Token::Time),
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
	use super::{OutputFormat, Token};

	#[test]
	fn parse_format() {
		let output = &[
			&OutputFormat::parse("String with {time} with some text ahead.").tokens,
			&OutputFormat::parse("}}{}{{}{}}}{{}{{}}}").tokens,
			&OutputFormat::parse("{time} text {time}").tokens,
		];
		let expected: &[&[Token]] = &[
			&[
				Token::Literal("String with ".into()),
				Token::Time,
				Token::Literal(" with some text ahead.".into())
			],
			&[Token::Literal("}}{}{{}{}}}{{}{{}}}".into())],
			&[
				Token::Time,
				Token::Literal(" text ".into()),
				Token::Time,
			]
		];
		assert_eq!(output, expected);
	}
}
