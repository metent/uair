mod app;
mod config;
mod socket;
mod session;
mod timer;

use std::env;
use std::io;
use uair::get_socket_path;
use argh::FromArgs;
use futures_lite::{FutureExt, StreamExt};
use signal_hook::consts::signal::*;
use signal_hook_async_std::Signals;
use crate::app::App;

fn main() {
	let args: Args = argh::from_env();
	let app = match App::new(args) {
		Ok(app) => app,
		Err(err) => { eprintln!("{}", err); return }
	};

	if let Err(err) = async_io::block_on(app.run().or(catch_term_signals())) {
		eprintln!("{}", err);
	}
}

#[derive(FromArgs)]
/// An extensible pomodoro timer
pub struct Args {
	/// specifies a config file.
	#[argh(option, short = 'c', default = "get_config_path()")]
	config: String,

	/// specifies a socket file.
	#[argh(option, short = 's', default = "get_socket_path()")]
	socket: String,
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

async fn catch_term_signals() -> Result<(), Error> {
	let mut signals = Signals::new(&[SIGTERM, SIGINT, SIGPIPE, SIGQUIT])?;
	signals.next().await;
	Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("IO Error: {0}")]
	IoError(#[from] io::Error),
	#[error("Config Error: {0}")]
	ConfError(#[from] toml::de::Error),
	#[error("Deserialization Error: {0}")]
	DeserError(#[from] bincode::Error),
}
