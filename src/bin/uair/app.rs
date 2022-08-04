use std::io::{self, Write};
use std::process;
use std::time::Duration;
use futures_lite::FutureExt;
use super::Args;
use super::server::Listener;
use super::timer::UairTimer;
use super::config::{Config, Session};

pub struct App {
	listener: Listener,
	ptr: SessionPointer,
	startup_text: String,
	pause_at_start: bool,
	done: bool,
}

impl App {
	pub fn new(args: Args, config: Config) -> anyhow::Result<Self> {
		Ok(App {
			listener: Listener::new(&args.socket)?,
			ptr: SessionPointer::new(config.sessions, config.loop_on_end).unwrap(),
			startup_text: config.startup_text,
			pause_at_start: config.pause_at_start,
			done: false,
		})
	}

	pub async fn run(mut self) -> anyhow::Result<()> {
		let mut stdout = io::stdout();
		write!(stdout, "{}", self.startup_text)?;
		stdout.flush()?;

		if self.pause_at_start { self.listener.wait(false, true, false).await?; }

		while !self.done {
			let mut timer = UairTimer::new(self.ptr.curr().duration, Duration::from_secs(1));

			if !self.ptr.curr().autostart {
				self.listener.wait(false, self.ptr.is_first(), self.ptr.is_last()).await?;
			}

			while self.run_session(&mut timer).await? {}

		}

		Ok(())
	}

	async fn run_session(&mut self, timer: &mut UairTimer) -> anyhow::Result<bool> {
		let (curr, first, last) = (self.ptr.curr(), self.ptr.is_first(), self.ptr.is_last());

		match timer.start(curr).or(self.listener.wait(true, first, last)).await? {
			Event::Pause => {
				timer.update_duration();
				self.wait().await
			}
			Event::Finished => {
				if !curr.command.is_empty() {
					let duration = humantime::format_duration(curr.duration).to_string();
					process::Command::new("sh")
						.env("name", &curr.name)
						.env("duration", duration)
						.arg("-c")
						.arg(&curr.command)
						.spawn()?;
				}
				if last { self.done = true };
				self.ptr.next();
				Ok(false)
			}
			Event::Next => {
				self.ptr.next();
				Ok(false)
			}
			Event::Prev => {
				self.ptr.prev();
				Ok(false)
			}
			_ => Ok(true)
		}
	}

	async fn wait(&mut self) -> anyhow::Result<bool> {
		let (first, last) = (self.ptr.is_first(), self.ptr.is_last());
		match self.listener.wait(false, first, last).await? {
			Event::Resume => Ok(true),
			Event::Next => {
				self.ptr.next();
				Ok(false)
			}
			Event::Prev => {
				self.ptr.prev();
				Ok(false)
			}
			_ => Ok(true)
		}
	}
}



struct SessionPointer {
	sessions: Vec<Session>,
	index: usize,
	loop_on_end: bool,
}

impl SessionPointer {
	fn new(sessions: Vec<Session>, loop_on_end: bool) -> Option<Self> {
		if sessions.len() == 0 { None }
		else { Some(SessionPointer { sessions, index: 0, loop_on_end }) }
	}

	fn curr(&self) -> &Session {
		&self.sessions[self.index]
	}

	fn next(&mut self) {
		if self.index < self.sessions.len() - 1 {
			self.index += 1;
		} else if self.loop_on_end {
			self.index = 0;
		}
	}

	fn prev(&mut self) {
		if self.index > 0 {
			self.index -= 1;
		} else if self.loop_on_end {
			self.index = self.sessions.len() - 1;
		}
	}

	fn is_last(&self) -> bool {
		self.index == self.sessions.len() - 1 && !self.loop_on_end
	}

	fn is_first(&self) -> bool {
		self.index == 0 && !self.loop_on_end
	}
}

pub enum Event {
	Pause,
	Resume,
	Finished,
	Next,
	Prev,
}
