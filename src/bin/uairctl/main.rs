use std::io::{self, Write};
use std::os::unix::net::UnixStream;
use uair::{Command, get_socket_path};
use argh::FromArgs;

fn main() -> Result<(), Error> {
	let args: Args = argh::from_env();

	let command = bincode::serialize(&args.command)?;
	let mut stream = UnixStream::connect(&args.socket)?;
	stream.write_all(&command)?;

	Ok(())
}

#[derive(FromArgs)]
/// An extensible pomodoro timer
struct Args {
	/// specifies the socket file.
	#[argh(option, short = 's', default = "get_socket_path()")]
	socket: String,

	#[argh(subcommand)]
	command: Command,
}

#[derive(Debug)]
enum Error {
	SerError(bincode::Error),
	IoError(io::Error),
}

impl From<bincode::Error> for Error{
	fn from(err: bincode::Error) -> Error {
		Error::SerError(err)
	}
}

impl From<io::Error> for Error{
	fn from(err: io::Error) -> Error {
		Error::IoError(err)
	}
}
