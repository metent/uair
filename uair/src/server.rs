use std::fs;
use std::path::PathBuf;
use smol::io::AsyncReadExt;
use smol::net::unix::UnixListener;
use super::Event;

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
		Ok(Event::Command(bincode::deserialize(&mut buffer)?))
	}
}

impl Drop for Listener {
	fn drop(&mut self) {
		_ = fs::remove_file(&self.path);
	}
}
