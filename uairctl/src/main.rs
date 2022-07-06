use std::env;
use std::os::unix::net::UnixStream;
use std::io::Write;
use common::Command;

fn main() {
	let mut args = env::args();
	args.next();
	let comm = if let Some(comm) = args.next() { comm } else { return };
	let comm = match &comm[..] {
		"resume" => Command::Resume,
		"pause" => Command::Pause,
		_ => return,
	};
	let mut stream = UnixStream::connect("/tmp/uair.sock").unwrap();
	let command = bincode::serialize(&comm).unwrap();
	stream.write_all(&command).unwrap();
}
