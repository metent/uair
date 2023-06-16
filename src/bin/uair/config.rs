use std::collections::HashMap;
use std::time::Duration;
use std::str::FromStr;
use serde::{Serialize, Deserialize};
use crate::session::{Color, Overridables, Session, Token, TimeFormatToken};

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
			sessions: self.sessions.into_iter().map(|s| s.build(&self.defaults)).collect(),
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
	#[serde(default = "Defaults::overrides")]
	overrides: HashMap<String, OverridablesBuilder>,
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
	fn overrides() -> HashMap<String, OverridablesBuilder> { HashMap::new() }
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
			overrides: Defaults::overrides(),
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
	#[serde(default)]
	overrides: HashMap<String, OverridablesBuilder>,
}

impl SessionBuilder {
	fn build(self, defaults: &Defaults) -> Session {
		let mut default_overrides = defaults.overrides.clone();
		default_overrides.extend(self.overrides);
		let overrides = default_overrides.into_iter().map(|(k, v)| {
			let default = defaults.overrides.get(&k);
			(k, v.build(default))
		}).collect();
		Session {
			name: self.name.unwrap_or_else(|| defaults.name.clone()),
			duration: self.duration.unwrap_or_else(|| defaults.duration.clone()),
			command: self.command.unwrap_or_else(|| defaults.command.clone()),
			format: self.format.map(|f| Token::parse(&f)).unwrap_or_else(|| Token::parse(&defaults.format)),
			time_format: TimeFormatToken::parse(self.time_format.as_ref().unwrap_or_else(|| &defaults.time_format)),
			autostart: self.autostart.unwrap_or_else(|| defaults.autostart.clone()),
			paused_state_text: self.paused_state_text.unwrap_or_else(|| defaults.paused_state_text.clone()),
			resumed_state_text: self.resumed_state_text.unwrap_or_else(|| defaults.resumed_state_text.clone()),
			overrides,
		}
	}
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

#[derive(Serialize, Deserialize, Default, Clone)]
struct OverridablesBuilder {
	format: Option<String>,
	time_format: Option<String>,
	paused_state_text: Option<String>,
	resumed_state_text: Option<String>,
}

impl OverridablesBuilder {
	fn build(self, defaults: Option<&OverridablesBuilder>) -> Overridables {
		let default_ob = OverridablesBuilder::default();
		let defaults = defaults.unwrap_or(&default_ob);
		Overridables {
			format: self.format.or(defaults.format.clone()).map(|f| Token::parse(&f)),
			time_format: self.time_format.or(defaults.time_format.clone()).map(|f| TimeFormatToken::parse(&f)),
			paused_state_text: self.paused_state_text.or(defaults.paused_state_text.clone()),
			resumed_state_text: self.resumed_state_text.or(defaults.resumed_state_text.clone()),
		}
	}
}
