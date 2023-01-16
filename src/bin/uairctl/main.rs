use std::io::{self, Read, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use uair::{Command, get_socket_path};
use argh::FromArgs;

fn main() -> Result<(), Error> {
	let args: Args = argh::from_env();

	let command = bincode::serialize(&args.command)?;
	let mut stream = UnixStream::connect(&args.socket)?;
	stream.write_all(&command)?;
	stream.shutdown(Shutdown::Write)?;

	if let Command::Fetch(_) = args.command {
		let mut buf = String::new();
		stream.read_to_string(&mut buf)?;

		write!(io::stdout(), "{}", buf)?;
	}

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

#[derive(thiserror::Error, Debug)]
enum Error {
	#[error("Serialization Error: {0}")]
	SerError(#[from] bincode::Error),
	#[error("Socket Connection Error: {0}")]
	IoError(#[from] io::Error),
}
