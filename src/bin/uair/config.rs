use std::time::Duration;
use std::str::FromStr;
use serde::{Serialize, Deserialize};
use crate::session::{Color, Session, Token, TimeFormatToken};

pub struct Config {
	pub iterations: Option<u64>,
	pub pause_at_start: bool,
	pub startup_text: String,
	pub sessions: Vec<Session>,
}

#[derive(Serialize, Deserialize)]
pub struct ConfigBuilder {
	#[serde(default)]
	loop_on_end: bool,
	iterations: Option<u64>,
	#[serde(default)]
	pause_at_start: bool,
	#[serde(default)]
	startup_text: String,
	#[serde(default)]
	defaults: Defaults,
	sessions: Vec<SessionBuilder>,
}

impl ConfigBuilder {
	pub fn deserialize(conf: &str) -> Result<Self, toml::de::Error> {
		toml::from_str(conf)
	}

	pub fn build(self) -> Config {
		Config {
			iterations: if self.loop_on_end && self.iterations != Some(0) {
				None
			} else if self.iterations.is_some() {
				self.iterations
			} else {
				Some(1)
			},
			pause_at_start: self.pause_at_start,
			startup_text: self.startup_text,
			sessions: self.sessions.into_iter().map(|s| Session {
				name: s.name.unwrap_or_else(|| self.defaults.name.clone()),
				duration: s.duration.unwrap_or_else(|| self.defaults.duration.clone()),
				command: s.command.unwrap_or_else(|| self.defaults.command.clone()),
				format: s.format.map(|f| Token::parse(&f)).unwrap_or_else(|| Token::parse(&self.defaults.format)),
				time_format: TimeFormatToken::parse(s.time_format.as_ref().unwrap_or_else(|| &self.defaults.time_format)),
				autostart: s.autostart.unwrap_or_else(|| self.defaults.autostart.clone()),
				paused_state_text: s.paused_state_text.unwrap_or_else(|| self.defaults.paused_state_text.clone()),
				resumed_state_text: s.resumed_state_text.unwrap_or_else(|| self.defaults.resumed_state_text.clone()),
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
	#[serde(default = "Defaults::format")]
	format: String,
	#[serde (default = "Defaults::time_format")]
	time_format: String,
	#[serde(default = "Defaults::autostart")]
	autostart: bool,
	#[serde(default = "Defaults::paused_state_text")]
	paused_state_text: String,
	#[serde(default = "Defaults::resumed_state_text")]
	resumed_state_text: String,
}

impl Defaults {
	fn name() -> String { "Work".into() }
	fn duration() -> Duration { Duration::from_secs(25 * 60) }
	fn command() -> String { "notify-send 'Session Completed!'".into() }
	fn format() -> String { "{time}\n".into() }
	fn time_format() -> String { "%*-Yyear%P %*-Bmonth%P %*-Dday%P %*-Hh %*-Mm %*-Ss".into() }
	fn autostart() -> bool { false }
	fn paused_state_text() -> String { "⏸".into() }
	fn resumed_state_text() -> String { "⏵".into() }
}

impl Default for Defaults {
	fn default() -> Self {
		Defaults {
			name: Defaults::name(),
			duration: Defaults::duration(),
			command: Defaults::command(),
			format: Defaults::format(),
			time_format: Defaults::time_format(),
			autostart: Defaults::autostart(),
			paused_state_text: Defaults::paused_state_text(),
			resumed_state_text: Defaults::resumed_state_text(),
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
	format: Option<String>,
	time_format: Option<String>,
	autostart: Option<bool>,
	paused_state_text: Option<String>,
	resumed_state_text: Option<String>,
}

impl FromStr for Token {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"{name}" => Ok(Token::Name),
			"{percent}" => Ok(Token::Percent),
			"{time}" => Ok(Token::Time),
			"{total}" => Ok(Token::Total),
			"{state}" => Ok(Token::State),
			"{black}" => Ok(Token::Color(Color::Black)),
			"{red}" => Ok(Token::Color(Color::Red)),
			"{green}" => Ok(Token::Color(Color::Green)),
			"{yellow}" => Ok(Token::Color(Color::Yellow)),
			"{blue}" => Ok(Token::Color(Color::Blue)),
			"{purple}" => Ok(Token::Color(Color::Purple)),
			"{cyan}" => Ok(Token::Color(Color::Cyan)),
			"{white}" => Ok(Token::Color(Color::White)),
			"{end}" => Ok(Token::Color(Color::End)),
			_ => Err(()),
		}
	}
}
