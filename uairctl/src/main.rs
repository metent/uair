use std::env;
use std::os::unix::net::UnixStream;
use std::io::Write;
use common::Command;

fn main() -> anyhow::Result<()> {
	let mut args = env::args();
	args.next();
	let comm = if let Some(comm) = args.next() { comm } else { return Ok(()) };
	let comm = match &comm[..] {
		"resume" => Command::Resume,
		"pause" => Command::Pause,
		_ => return Ok(()),
	};
	let mut stream = UnixStream::connect("/tmp/uair.sock")?;
	let command = bincode::serialize(&comm)?;
	stream.write_all(&command)?;
	Ok(())
}
