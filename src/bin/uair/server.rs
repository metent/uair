use std::fs;
use std::path::PathBuf;
use async_net::unix::UnixListener;
use futures_lite::AsyncReadExt;
use uair::Command;
use super::app::Event;

pub struct Listener {
	path: PathBuf,
	listener: UnixListener,
}

impl Listener {
	pub fn new(path: &str) -> anyhow::Result<Listener> {
		Ok(Listener { path: path.into(), listener: UnixListener::bind(path)? })
	}

	async fn listen(&self) -> anyhow::Result<Command> {
		let (mut stream, _) = self.listener.accept().await?;
		let mut buffer = Vec::new();
		stream.read_to_end(&mut buffer).await?;
		Ok(bincode::deserialize(&mut buffer)?)
	}

	pub async fn wait(
		&self,
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
