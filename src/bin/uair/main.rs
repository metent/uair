mod app;
mod config;
mod server;
mod timer;

use std::env;
use futures_lite::{FutureExt, StreamExt};
use signal_hook::consts::signal::*;
use signal_hook_async_std::Signals;
use crate::config::UairConfig;

fn main() -> anyhow::Result<()> {
	let args = Args::args()?;
	if args.help {
		println!("{}", Args::help());
		return Ok(());
	}
	let config = UairConfig::get(&args)?;
	async_io::block_on(app::run(args, config).or(handle_signals()))?;
	Ok(())
}

argwerk::define! {
	/// An extensible pomodoro timer.
	#[usage = "uair [options..]"]
	pub struct Args {
		config_path: String = if let Ok(xdg_config_home) = env::var("XDG_CONFIG_HOME") {
			xdg_config_home + "/uair/uair.toml"
		} else if let Ok(home) = env::var("HOME") {
			home + "/.config/uair/uair.toml"
		} else {
			"~/.config/uair/uair.toml".into()
		},
		socket_path: String = if let Ok(xdg_runtime_dir) = env::var("XDG_RUNTIME_DIR") {
			xdg_runtime_dir + "/uair.sock"
		} else if let Ok(tmp_dir) = env::var("TMPDIR") {
			tmp_dir + "/uair.sock"
		} else {
			"/tmp/uair.sock".into()
		},
		help: bool,
	}
	/// Specifies a config file.
	["-c" | "--config", path] => {
		config_path = path;
	}
	/// Specifies a socket file.
	["-s" | "--socket", path] => {
		socket_path = path;
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
