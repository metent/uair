use std::time::Duration;
use config::{Config, File, FileFormat};
use serde::{Serialize, Deserialize};
use super::Args;

#[derive(Serialize, Deserialize)]
pub struct UairConfig {
	pub sessions: Vec<SessionConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct SessionConfig {
	pub name: String,
	#[serde(with = "humantime_serde")]
	pub duration: Duration,
	pub command: String,
	pub before: String,
	pub after: String,
}

pub fn get(args: Args) -> anyhow::Result<UairConfig> {
	Ok(Config::builder()
		.add_source(File::new(&args.config_path, FileFormat::Ron))
		.build()?
		.try_deserialize::<UairConfig>()?
	)
}
