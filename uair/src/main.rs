mod config;
mod server;
mod timer;

use std::time::Duration;
use futures_lite::{FutureExt, StreamExt};
use signal_hook::consts::signal::*;
use signal_hook_async_std::Signals;
use common::Command;
use crate::server::Listener;
use crate::timer::UairTimer;
use crate::config::UairConfig;

fn main() -> anyhow::Result<()> {
	let args = Args::args()?;
	if args.help {
		println!("{}", Args::help());
		return Ok(());
	}
	let config = config::get(args)?;
	smol::block_on(amain(config).or(handle_signals()))?;
	Ok(())
}

argwerk::define! {
	/// An extensible pomodoro timer.
	#[usage = "uair [OPTION]..."]
	pub struct Args {
		config_path: String,
		help: bool,
	}
	/// Path to config file.
	["-c" | "--config", path] => {
		config_path = path;
	}
	/// Print this help.
	["-h" | "--help"] => {
		help = true;
	}
}

impl Default for Args {
	fn default() -> Self {
		Args {
			config_path: "~/.config/uair/uair.ron".into(),
			help: false,
		}
	}
}

async fn amain(config: UairConfig) -> anyhow::Result<()> {
	let listener = Listener::new("/tmp/uair.sock")?;
	for session in config.sessions {
		let mut timer = UairTimer::new(
			session.duration,
			Duration::from_secs(1),
			session.before,
			session.after
		);
		loop {
			match timer.start().or(listener.listen()).await? {
				Event::Command(Command::Pause | Command::Toggle) => {
					timer.update_duration();
					loop {
						match listener.listen().await? {
							Event::Command(Command::Resume | Command::Toggle) => break,
							_ => {}
						}
					}
				}
				Event::Finished => break,
				_ => {}
			}
		}
	}

	Ok(())
}

pub enum Event {
	Command(Command),
	Finished,
}

async fn handle_signals() -> anyhow::Result<()> {
	let mut signals = Signals::new(&[SIGTERM, SIGINT, SIGQUIT])?;
	while let Some(signal) = signals.next().await {
		match signal {
			SIGTERM | SIGINT | SIGQUIT => break,
			_ => {},
		}
	}
	Ok(())
}
