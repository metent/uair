use std::fs;
use std::path::PathBuf;
use smol::io::AsyncReadExt;
use smol::net::unix::UnixListener;
use common::Command;

pub struct Listener {
	path: PathBuf,
	listener: UnixListener,
}

impl Listener {
	pub fn new(path: &str) -> anyhow::Result<Listener> {
		Ok(Listener { path: path.into(), listener: UnixListener::bind(path)? })
	}

	pub async fn listen(&self) -> anyhow::Result<Event> {
		let (mut stream, _) = self.listener.accept().await?;
		let mut buffer = Vec::new();
		stream.read_to_end(&mut buffer).await?;
		match bincode::deserialize::<Command>(&mut buffer)? {
			Command::Pause => Ok(Event::Pause),
			Command::Resume => Ok(Event::Start),
		}
	}
}

impl Drop for Listener {
	fn drop(&mut self) {
		_ = fs::remove_file(&self.path);
	}
}

pub enum Event {
	Pause,
	Stop,
	Start,
}
