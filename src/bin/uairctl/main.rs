use std::os::unix::net::UnixStream;
use std::io::Write;
use gumdrop::Options;
use serde::{Serialize, Deserialize};
use uair::get_socket_path;

fn main() -> anyhow::Result<()> {
	let mut args = Args::parse_args_default_or_exit();

	let comm = if let Some(comm) = args.command { comm } else { return Ok(()) };
	if args.socket.is_empty() { args.socket = get_socket_path() };

	let command = bincode::serialize(&comm)?;

	let mut stream = UnixStream::connect(&args.socket)?;
	stream.write_all(&command)?;

	Ok(())
}

#[derive(Options)]
struct Args {
	#[options(help = "Show help message and quit.")]
	help: bool,
	#[options(help = "Specifies the socket file.")]
	socket: String,
	#[options(command)]
	command: Option<Command>,
}

#[derive(Options, Serialize, Deserialize)]
enum Command {
	#[options(help = "Pause the timer.")]
	Pause(NoArgs),
	#[options(help = "Resume the timer.")]
	Resume(NoArgs),
	#[options(help = "Toggle the state of the timer.")]
	Toggle(NoArgs),
	#[options(help = "Jump to the next session.")]
	Next(NoArgs),
	#[options(help = "Jump to the previous session")]
	Prev(NoArgs),
}

#[derive(Options, Serialize, Deserialize)]
struct NoArgs {}
