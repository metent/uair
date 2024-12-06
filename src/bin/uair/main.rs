mod app;
mod config;
mod session;
mod socket;
mod timer;

use crate::app::App;
use argh::FromArgs;
use futures_lite::{FutureExt, StreamExt};
use log::{error, LevelFilter};
use signal_hook::consts::signal::*;
use signal_hook_async_std::Signals;
use simplelog::{ColorChoice, Config as LogConfig, TermLogger, TerminalMode, WriteLogger};
use std::env;
use std::fmt::Display;
use std::fs::File;
use std::io::{self, Write};
use std::process::ExitCode;
use uair::get_socket_path;

fn main() -> ExitCode {
	let args: Args = argh::from_env();
	if args.version {
		_ = writeln!(
			io::stdout(),
			"{} version {}",
			env!("CARGO_PKG_NAME"),
			env!("CARGO_PKG_VERSION"),
		);
		return ExitCode::SUCCESS;
	}

	let enable_stderr = args.log != "-";

	if let Err(err) = init_logger(&args) {
		return raise_err(err, enable_stderr);
	}

	let app = match App::new(args) {
		Ok(app) => app,
		Err(err) => {
			return raise_err(err, enable_stderr);
		}
	};

	if let Err(err) = async_io::block_on(app.run().or(catch_term_signals())) {
		return raise_err(err, enable_stderr);
	}

	ExitCode::SUCCESS
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

	/// specifies a log file.
	#[argh(option, short = 'l', default = "\"-\".into()")]
	log: String,

	/// run without writing to standard output.
	#[argh(switch, short = 'q')]
	quiet: bool,

	/// display version number and then exit.
	#[argh(switch, short = 'v')]
	version: bool,
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
	let mut signals = Signals::new([SIGTERM, SIGINT, SIGQUIT])?;
	signals.next().await;
	Ok(())
}

fn init_logger(args: &Args) -> Result<(), Error> {
	if args.log == "-" {
		TermLogger::init(
			LevelFilter::Info,
			LogConfig::default(),
			TerminalMode::Stderr,
			ColorChoice::Auto,
		)?;
	} else {
		WriteLogger::init(
			LevelFilter::Info,
			LogConfig::default(),
			File::create(&args.log)?,
		)?;
	}
	Ok(())
}

fn raise_err(err: impl Display, enable_stderr: bool) -> ExitCode {
	error!("{}", err);
	if enable_stderr {
		eprintln!("{}", err)
	}
	ExitCode::FAILURE
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("Log Error: {0}")]
	LogError(#[from] log::SetLoggerError),
	#[error("IO Error: {0}")]
	IoError(#[from] io::Error),
	#[error("Config Error: {0}")]
	ConfError(#[from] toml::de::Error),
	#[error("Deserialization Error: {0}")]
	DeserError(#[from] bincode::Error),
}
