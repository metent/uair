use std::io;
use std::fs;
use std::path::PathBuf;
use async_net::unix::UnixListener;
use futures_lite::AsyncReadExt;

pub struct Listener {
	path: PathBuf,
	listener: UnixListener,
}

impl Listener {
	pub fn new(path: &str) -> io::Result<Listener> {
		Ok(Listener { path: path.into(), listener: UnixListener::bind(path)? })
	}

	pub async fn listen(&self) -> io::Result<[u8; 4]> {
		let (mut stream, _) = self.listener.accept().await?;
		let mut buffer = [0; 4];
		stream.read(&mut buffer).await?;
		Ok(buffer)
	}
}

impl Drop for Listener {
	fn drop(&mut self) {
		_ = fs::remove_file(&self.path);
	}
}
