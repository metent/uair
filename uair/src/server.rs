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
	pub fn new(path: &str) -> Self {
		Listener { path: path.into(), listener: UnixListener::bind(path).unwrap() }
	}

	pub async fn listen(&self) -> Event {
		let (mut stream, _) = self.listener.accept().await.unwrap();
		let mut buffer = Vec::new();
		stream.read_to_end(&mut buffer).await.unwrap();
		match bincode::deserialize::<Command>(&mut buffer).unwrap() {
			Command::Pause => Event::Pause,
			Command::Resume => Event::Start,
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
