use std::io::{self, Write};
use std::fs;
use std::path::PathBuf;
use async_net::unix::{UnixListener, UnixStream};
use futures_lite::{AsyncReadExt, AsyncWriteExt};
use futures_lite::io::BlockOn;

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
	pub async fn read<'buf>(&mut self, buffer: &'buf mut Vec<u8>) -> io::Result<&'buf [u8]> {
		let n_bytes = self.stream.read_to_end(buffer).await?;
		Ok(&buffer[..n_bytes])
	}

	pub async fn write(&mut self, data: &[u8]) -> io::Result<()> {
		self.stream.write_all(data).await?;
		Ok(())
	}

	pub fn into_blocking(self) -> BlockingStream {
		BlockingStream { stream: BlockOn::new(self.stream) }
	}
}

pub struct BlockingStream {
	stream: BlockOn<UnixStream>,
}

impl BlockingStream {
	pub fn write(&mut self, data: &[u8]) -> io::Result<()> {
		self.stream.write_all(data)?;
		self.stream.flush()?;
		Ok(())
	}
}
