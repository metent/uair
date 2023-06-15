use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::str;
use uair::{Command, FetchArgs, get_socket_path};
use argh::FromArgs;

fn main() -> Result<(), Error> {
	let mut args: Args = argh::from_env();
	if let Command::Fetch(FetchArgs { format }) = &mut args.command {
		*format = unescape(&format);
	}

	let command = bincode::serialize(&args.command)?;
	let mut stream = UnixStream::connect(&args.socket)?;

	stream.write_all(&command)?;
	stream.shutdown(Shutdown::Write)?;

	match args.command {
		Command::Fetch(_) => {
			let mut buf = String::new();
			stream.read_to_string(&mut buf)?;

			write!(io::stdout(), "{}", buf)?;
		}
		Command::Listen(_) => {
			let mut reader = BufReader::new(stream);
			let mut buf = Vec::new();

			loop {
				reader.read_until(b'\0', &mut buf)?;
				if buf.is_empty() { break; }
				write!(io::stdout(), "{}", str::from_utf8(&buf)?)?;
				buf.clear();
			}
		}
		_ => {}
	}

	Ok(())
}

fn unescape(input: &str) -> String {
	let mut res = String::new();
	let mut chars = input.char_indices();
	'outer: while let Some((_, c)) = chars.next() {
		if c != '\\' {
			res.push(c);
		} else if let Some((i, c)) = chars.next() {
			match c {
				'b' => res.push('\u{0008}'),
				'f' => res.push('\u{000c}'),
				'n' => res.push('\n'),
				'r' => res.push('\r'),
				't' => res.push('\t'),
				'\"' => res.push('\"'),
				'\\' => res.push('\\'),
				'u' => {
					for _ in 0..4 {
						if chars.next().is_none() {
							res.push_str(&input[i - 1..]);
							break 'outer;
						}
					}
					match u32::from_str_radix(&input[i + 1..i + 5], 16)
						.map(|num| char::from_u32(num)) {
						Ok(Some(num)) => res.push(num),
						_ => res.push_str(&input[i - 1..i + 5]),
					}
				}
				'U' => {
					for _ in 0..8 {
						if chars.next().is_none() {
							res.push_str(&input[i - 1..]);
							break 'outer;
						}
					}
					match u32::from_str_radix(&input[i + 1..i + 9], 16)
						.map(|num| char::from_u32(num)) {
						Ok(Some(num)) => res.push(num),
						_ => res.push_str(&input[i - 1..i + 9]),
					}
				}
				_ => res.push_str(&input[i - 1..i + 1]),
			}
		} else {
			res.push('\\');
			break;
		}
	}

	res
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
	#[error("UTF8 Error: {0}")]
	Utf8Error(#[from] str::Utf8Error),
}

#[cfg(test)]
mod tests {
	use super::unescape;

	#[test]
	fn unescape_test() {
		assert_eq!(unescape(r"\u0000"), "\0");
		assert_eq!(unescape(r"\u0009"), "\t");
		assert_eq!(unescape(r"\u000a"), "\n");
		assert_eq!(unescape(r"\uffff"), "\u{ffff}");
		assert_eq!(unescape(r"\u0000Foo"), "\0Foo");

		assert_eq!(unescape(r"\nFoo"), "\nFoo");
		assert_eq!(unescape(r"Foo\"), "Foo\\");
	}
}
