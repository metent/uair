use std::fs;
use std::time::Duration;
use serde::{Serialize, Deserialize};
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
	#[serde(default = "Defaults::autostart")]
	autostart: bool,
}

impl Defaults {
	fn name() -> String { "Work".into() }
	fn duration() -> Duration { Duration::from_secs(25 * 60) }
	fn command() -> String { "notify-send 'Session Completed!'".into() }
	fn before() -> String { "".into() }
	fn after() -> String { "\n".into() }
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
	autostart: Option<bool>,
}
