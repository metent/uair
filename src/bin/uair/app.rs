use std::io::{self, Write};
use std::fs;
use std::process;
use std::time::Duration;
use uair::{Command, PauseArgs, ResumeArgs};
use futures_lite::FutureExt;
use crate::{Args, Error};
use crate::config::{Config, ConfigBuilder};
use crate::socket::Listener;
use crate::session::SessionId;
use crate::timer::UairTimer;

pub struct App {
	listener: Listener,
	sid: SessionId,
	config: Config,
}

impl App {
	pub fn new(args: Args) -> Result<Self, Error> {
		let conf_data = fs::read_to_string(&args.config)?;
		let config = ConfigBuilder::deserialize(&conf_data)?.build();
		Ok(App {
			listener: Listener::new(&args.socket)?,
			sid: SessionId::new(&config.sessions, config.iterations),
			config,
		})
	}

	pub async fn run(mut self) -> Result<(), Error> {
		let mut stdout = io::stdout();
		write!(stdout, "{}", self.config.startup_text)?;
		stdout.flush()?;

		let session = &self.config.sessions[self.sid.curr()];
		let mut timer = UairTimer::new(session.duration, Duration::from_secs(1));
		let mut state = if self.config.pause_at_start || !session.autostart
			{ State::Paused } else { State::Resumed };

		loop {
			match state {
				State::Paused => state = self.pause_session().await?,
				State::Resumed => state = self.run_session(&mut timer).await?,
				State::Finished => break,
				State::Reset => {
					let session = &self.config.sessions[self.sid.curr()];
					timer = UairTimer::new(session.duration, Duration::from_secs(1));
					state = if session.autostart { State::Resumed } else { State::Paused };
				}
			}
		}
		Ok(())
	}

	async fn run_session(&mut self, timer: &mut UairTimer) -> Result<State, Error> {
		let session = &self.config.sessions[self.sid.curr()];

		match timer.start(session).or(self.handle_commands::<true>()).await? {
			Event::Finished => {
				if !session.command.is_empty() {
					let duration = humantime::format_duration(session.duration).to_string();
					process::Command::new("sh")
						.env("name", &session.name)
						.env("duration", duration)
						.arg("-c")
						.arg(&session.command)
						.spawn()?;
				}
				if self.sid.is_last() { return Ok(State::Finished) };
				self.sid.next();
			}
			Event::Command(Command::Pause(_)) => {
				timer.update_duration();
				return Ok(State::Paused);
			}
			Event::Command(Command::Next(_)) => self.sid.next(),
			Event::Command(Command::Prev(_)) => self.sid.prev(),
			_ => unreachable!(),
		}
		Ok(State::Reset)
	}

	async fn pause_session(&mut self) -> Result<State, Error> {
		match self.handle_commands::<false>().await? {
			Event::Command(Command::Resume(_)) => return Ok(State::Resumed),
			Event::Command(Command::Next(_)) => self.sid.next(),
			Event::Command(Command::Prev(_)) => self.sid.prev(),
			_ => unreachable!(),
		}
		Ok(State::Reset)
	}

	async fn handle_commands<const R: bool>(&self) -> Result<Event, Error> {
		loop {
			let msg = self.listener.listen().await?;
			let command: Command = bincode::deserialize(&msg)?;
			match command {
				Command::Pause(_) | Command::Toggle(_) if R =>
					return Ok(Event::Command(Command::Pause(PauseArgs {}))),
				Command::Resume(_) | Command::Toggle(_) if !R =>
					return Ok(Event::Command(Command::Resume(ResumeArgs {}))),
				Command::Next(_) if !self.sid.is_last() =>
					return Ok(Event::Command(command)),
				Command::Prev(_) if !self.sid.is_first() =>
					return Ok(Event::Command(command)),
				_ => {}
			}
		}
	}
}

pub enum Event {
	Command(Command),
	Finished,
}

enum State {
	Paused,
	Resumed,
	Finished,
	Reset,
}
