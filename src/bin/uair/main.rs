mod app;
mod config;
mod server;
mod timer;

use std::env;
use futures_lite::{FutureExt, StreamExt};
use signal_hook::consts::signal::*;
use signal_hook_async_std::Signals;

fn main() -> anyhow::Result<()> {
	let args = Args::args()?;
	if args.help {
		println!("{}", Args::help());
		return Ok(());
	}
	let config = config::get(args)?;
	smol::block_on(app::run(config).or(handle_signals()))?;
	Ok(())
}

argwerk::define! {
	/// An extensible pomodoro timer.
	#[usage = "uair [options..]"]
	pub struct Args {
		config_path: String = env::var("HOME").unwrap_or("/root".into()) +
			"/.config/uair/uair.toml",
		help: bool,
	}
	/// Specifies a config file.
	["-c" | "--config", path] => {
		config_path = path;
	}
	/// Show help message and quit.
	["-h" | "--help"] => {
		help = true;
	}
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
