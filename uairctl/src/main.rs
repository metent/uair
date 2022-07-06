use std::env;
use std::os::unix::net::UnixStream;
use std::io::Write;
use common::Command;

argwerk::define! {
	/// An extensible pomodoro timer.
	#[derive(Default)]
	#[usage = "uair [OPTION]..."]
	struct Args {
		help: bool,
		pause: bool,
		resume: bool,
	}
	/// Pause the timer.
	["-p" | "--pause"] => {
		pause = true;
	}
	/// Resume the timer.
	["-r" | "--resume"] => {
		resume = true;
	}
	/// Print this help.
	["-h" | "--help"] => {
		help = true;
	}
}

fn main() -> anyhow::Result<()> {
	let args = Args::args()?;

	if args.help {
		println!("{}", Args::help());
		return Ok(())
	};

	let comm = match (args.pause, args.resume) {
		(true, true) => Command::Toggle,
		(true, false) => Command::Pause,
		(false, true) => Command::Resume,
		(false, false) => {
			println!("{}", Args::help());
			return Ok(());
		}
	};

	let mut stream = UnixStream::connect("/tmp/uair.sock")?;
	let command = bincode::serialize(&comm)?;
	stream.write_all(&command)?;
	Ok(())
}
