use std::fs;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use super::Args;

#[derive(Serialize, Deserialize)]
pub struct UairConfig {
	#[serde(default)]
	pub loop_on_end: bool,
	#[serde(default)]
	pub pause_at_start: bool,
	#[serde(default)]
	pub startup_text: String,
	#[serde(default)]
	pub defaults: Defaults,
	pub sessions: Vec<SessionConfig>,
}

impl UairConfig {
	pub fn get(args: &Args) -> anyhow::Result<Self> {
		Ok(toml::from_str(&fs::read_to_string(&args.config)?)?)
	}

	pub fn name(&self, i: usize) -> &str {
		match &self.sessions[i].name {
			Some(name) => &name,
			None => &self.defaults.name,
		}
	}

	pub fn duration(&self, i: usize) -> Duration {
		match &self.sessions[i].duration {
			Some(duration) => *duration,
			None => self.defaults.duration,
		}
	}

	pub fn command(&self, i: usize) -> &str {
		match &self.sessions[i].command {
			Some(command) => &command,
			None => &self.defaults.command,
		}
	}

	pub fn before(&self, i: usize) -> &str {
		match &self.sessions[i].before {
			Some(before) => &before,
			None => &self.defaults.before,
		}
	}

	pub fn after(&self, i: usize) -> &str {
		match &self.sessions[i].after {
			Some(after) => &after,
			None => &self.defaults.after,
		}
	}

	pub fn autostart(&self, i: usize) -> bool {
		match &self.sessions[i].autostart {
			Some(autostart) => *autostart,
			None => self.defaults.autostart,
		}
	}

	pub fn nb_sessions(&self) -> usize {
		self.sessions.len()
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
pub struct SessionConfig {
	name: Option<String>,
	#[serde(with = "humantime_serde")]
	#[serde(default)]
	duration: Option<Duration>,
	command: Option<String>,
	before: Option<String>,
	after: Option<String>,
	autostart: Option<bool>,
}
