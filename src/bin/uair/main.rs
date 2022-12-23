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

fn main() -> Result<(), Error> {
	let args: Args = argh::from_env();

	async_io::block_on(App::new(args)?.run().or(catch_term_signals()))
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

#[derive(Debug)]
pub enum Error {
	IoError(io::Error),
	ConfError(toml::de::Error),
	DeserError(bincode::Error),
}

impl From<io::Error> for Error {
	fn from(err: io::Error) -> Error {
		Error::IoError(err)
	}
}

impl From<toml::de::Error> for Error {
	fn from(err: toml::de::Error) -> Error {
		Error::ConfError(err)
	}
}

impl From<bincode::Error> for Error {
	fn from(err: bincode::Error) -> Error {
		Error::DeserError(err)
	}
}
