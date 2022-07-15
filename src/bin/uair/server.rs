use std::fs;
use std::path::PathBuf;
use async_net::unix::UnixListener;
use futures_lite::AsyncReadExt;
use uair::Command;
use super::app::Event;

pub struct Listener {
	path: PathBuf,
	listener: UnixListener,
	buffer: Vec<u8>,
}

impl Listener {
	pub fn new(path: &str) -> anyhow::Result<Listener> {
		Ok(Listener { path: path.into(), listener: UnixListener::bind(path)?, buffer: Vec::new() })
	}

	async fn listen(&mut self) -> anyhow::Result<Command> {
		let (mut stream, _) = self.listener.accept().await?;
		self.buffer.clear();
		stream.read_to_end(&mut self.buffer).await?;
		Ok(bincode::deserialize(&mut self.buffer)?)
	}

	pub async fn wait(
		&mut self,
		running: bool,
		disable_prev: bool,
		disable_next: bool
	) -> anyhow::Result<Event> {
		loop {
			match self.listen().await? {
				Command::Pause | Command::Toggle if running => return Ok(Event::Pause),
				Command::Resume | Command::Toggle if !running => return Ok(Event::Resume),
				Command::Next if !disable_next => return Ok(Event::Next),
				Command::Prev if !disable_prev => return Ok(Event::Prev),
				_ => {},
			}
		}
	}
}

impl Drop for Listener {
	fn drop(&mut self) {
		_ = fs::remove_file(&self.path);
	}
}
