use std::io;
use std::fs;
use std::path::PathBuf;
use async_net::unix::{UnixListener, UnixStream};
use futures_lite::AsyncReadExt;

pub struct Listener {
	path: PathBuf,
	listener: UnixListener,
}

impl Listener {
	pub fn new(path: &str) -> io::Result<Listener> {
		Ok(Listener { path: path.into(), listener: UnixListener::bind(path)? })
	}

	pub async fn listen(&self) -> io::Result<Stream> {
		let (stream, _) = self.listener.accept().await?;
		Ok(Stream { stream })
	}
}

impl Drop for Listener {
	fn drop(&mut self) {
		_ = fs::remove_file(&self.path);
	}
}

pub struct Stream {
	stream: UnixStream,
}

impl Stream {
	pub async fn read(&mut self) -> io::Result<[u8; 4]> {
		let mut buffer = [0; 4];
		self.stream.read(&mut buffer).await?;
		Ok(buffer)
	}
}
