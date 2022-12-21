mod app;
mod config;
mod server;
mod timer;

use std::env;
use futures_lite::{FutureExt, StreamExt};
use gumdrop::Options;
use signal_hook::consts::signal::*;
use signal_hook_async_std::Signals;
use uair::get_socket_path;
use crate::app::App;

fn main() -> anyhow::Result<()> {
	let mut args = Args::parse_args_default_or_exit();
	if args.config.is_empty() { args.config = get_config_path() }
	if args.socket.is_empty() { args.socket = get_socket_path() }

	async_io::block_on(App::new(args)?.run().or(handle_signals()))?;
	Ok(())
}

#[derive(Options)]
pub struct Args {
	#[options(help = "Show help message and quit.")]
	help: bool,
	#[options(help = "Specifies a config file.")]
	config: String,
	#[options(help = "Specifies a socket file.")]
	socket: String,
}

async fn handle_signals() -> anyhow::Result<()> {
	let mut signals = Signals::new(&[SIGTERM, SIGINT, SIGPIPE, SIGQUIT])?;
	while let Some(signal) = signals.next().await {
		match signal {
			SIGTERM | SIGINT | SIGPIPE | SIGQUIT => break,
			_ => {},
		}
	}
	Ok(())
}

fn get_config_path() -> String {
	if let Ok(xdg_config_home) = env::var("XDG_CONFIG_HOME") {
		xdg_config_home + "/uair/uair.toml"
	} else if let Ok(home) = env::var("HOME") {
		home + "/.config/uair/uair.toml"
	} else {
		"~/.config/uair/uair.toml".into()
	}
}
