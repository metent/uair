use std::env;
use std::os::unix::net::UnixStream;
use std::io::Write;
use uair::Command;

argwerk::define! {
	/// Command-line interface for controlling uair.
	#[derive(Default)]
	#[usage = "uairctl [options..]"]
	struct Args {
		pause: bool,
		resume: bool,
		socket_path: String = if let Ok(xdg_runtime_dir) = env::var("XDG_RUNTIME_DIR") {
			xdg_runtime_dir + "/uair.sock"
		} else if let Ok(tmp_dir) = env::var("TMPDIR") {
			tmp_dir + "/uair.sock"
		} else {
			"/tmp/uair.sock".into()
		},
		help: bool,
	}
	/// Pause the timer.
	["-p" | "--pause"] => {
		pause = true;
	}
	/// Resume the timer.
	["-r" | "--resume"] => {
		resume = true;
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
	let command = bincode::serialize(&comm)?;

	let mut stream = UnixStream::connect(&args.socket_path)?;
	stream.write_all(&command)?;

	Ok(())
}
