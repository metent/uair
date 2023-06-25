use std::io::{self, Write, Error as IoError, ErrorKind};
use std::fs;
use std::time::{Duration, Instant};
use uair::{Command, FetchArgs, ListenArgs, PauseArgs, ResumeArgs, JumpArgs};
use futures_lite::FutureExt;
use crate::{Args, Error};
use crate::config::{Config, ConfigBuilder};
use crate::socket::{Listener, Stream};
use crate::session::{Session, SessionId, Token};
use crate::timer::UairTimer;

pub struct App {
	data: AppData,
	state: State,
	timer: UairTimer,
}

impl App {
	pub fn new(args: Args) -> Result<Self, Error> {
		let data = AppData::new(args)?;
		Ok(App {
			data,
			state: State::Paused(Duration::ZERO),
			timer: UairTimer::new(Duration::from_secs(1)),
		})
	}

	pub async fn run(mut self) -> Result<(), Error> {
		self.start_up().await?;
		loop {
			match self.state {
				State::Paused(duration) => self.pause_session(duration).await?,
				State::Resumed(start, dest) => self.run_session(start, dest).await?,
				State::Finished => break,
			}
		}
		Ok(())
	}

	async fn start_up(&mut self) -> Result<(), Error> {
		let mut stdout = io::stdout();
		write!(stdout, "{}", self.data.config.startup_text)?;
		stdout.flush()?;

		if !self.data.config.pause_at_start { return Ok(()); }

		loop {
			match self.data.handle_commands::<false>().await? {
				Event::Finished | Event::Command(Command::Resume(_) | Command::Next(_)) => {
					self.state = self.data.initial_state();
					break;
				}
				Event::Command(Command::Prev(_)) => {},
				Event::Jump(idx) => { self.state = self.data.initial_jump(idx); break; }
				Event::Command(Command::Reload(_)) => self.data.read_conf::<true>()?,
				Event::Fetch(format, stream) =>
					self.data.handle_fetch_paused(format, stream, Duration::ZERO).await?,
				Event::Listen(overrid, stream) => self.timer.add_stream(stream.into_blocking(), overrid),
				_ => unreachable!(),
			}
		}

		Ok(())
	}

	async fn run_session(&mut self, start: Instant, dest: Instant) -> Result<(), Error> {
		let session = &self.data.curr_session();

		match self.timer.start(session, start, dest).or(self.data.handle_commands::<true>()).await? {
			Event::Finished => {
				session.run_command()?;
				if self.data.sid.is_last() {
					self.state = State::Finished;
				} else {
					self.state = self.data.next_session();
				}
			}
			Event::Command(Command::Pause(_)) => self.state = State::Paused(dest - Instant::now()),
			Event::Command(Command::Next(_)) => self.state = self.data.next_session(),
			Event::Command(Command::Prev(_)) => self.state = self.data.prev_session(),
			Event::Jump(idx) => self.state = self.data.jump_session(idx),
			Event::Command(Command::Reload(_)) => {
				self.data.read_conf::<true>()?;
				self.state = State::Resumed(Instant::now(), dest);
			}
			Event::Fetch(format, stream) =>
				self.data.handle_fetch_resumed(format, stream, dest).await?,
			Event::Listen(overrid, stream) => {
				self.timer.add_stream(stream.into_blocking(), overrid);
				self.state = State::Resumed(Instant::now(), dest);
			}
			_ => unreachable!(),
		}
		Ok(())
	}

	async fn pause_session(&mut self, duration: Duration) -> Result<(), Error> {
		const DELTA: Duration = Duration::from_nanos(1_000_000_000 - 1);
		let session = self.data.curr_session();

		self.timer.write::<false>(session, duration + DELTA)?;

		match self.data.handle_commands::<false>().await? {
			Event::Finished => {
				session.run_command()?;
				if self.data.sid.is_last() {
					self.state = State::Finished;
				} else {
					self.state = self.data.next_session();
				}
			}
			Event::Command(Command::Resume(_)) => {
				self.timer.write::<true>(session, duration + DELTA)?;
				let start = Instant::now();
				self.state = State::Resumed(start, start + duration);
			}
			Event::Command(Command::Next(_)) => self.state = self.data.next_session(),
			Event::Command(Command::Prev(_)) => self.state = self.data.prev_session(),
			Event::Jump(idx) => self.state = self.data.jump_session(idx),
			Event::Command(Command::Reload(_)) => self.data.read_conf::<true>()?,
			Event::Fetch(format, stream) =>
				self.data.handle_fetch_paused(format, stream, duration + DELTA).await?,
			Event::Listen(overrid, stream) =>
				self.timer.add_stream(stream.into_blocking(), overrid),
			_ => unreachable!(),
		}
		Ok(())
	}
}

pub enum Event {
	Command(Command),
	Jump(usize),
	Fetch(String, Stream),
	Finished,
	Listen(Option<String>, Stream),
}

struct AppData {
	listener: Listener,
	sid: SessionId,
	config: Config,
	config_path: String,
}

impl AppData {
	fn new(args: Args) -> Result<Self, Error> {
		let mut data = AppData {
			listener: Listener::new(&args.socket)?,
			sid: SessionId::default(),
			config: Config::default(),
			config_path: args.config,
		};
		data.read_conf::<false>()?;
		Ok(data)
	}

	fn read_conf<const R: bool>(&mut self) -> Result<(), Error> {
		let conf_data = fs::read_to_string(&self.config_path).map_err(|_|
			Error::IoError(IoError::new(
				ErrorKind::NotFound,
				format!("Could not load config file \"{}\"", self.config_path),
			)
		))?;
		let config = ConfigBuilder::deserialize(&conf_data)?.build()?;
		let mut sid = SessionId::new(&config.sessions, config.iterations);

		if R {
			let curr_id = &self.curr_session().id;
			if let Some(&idx) = config.idmap.get(curr_id) {
				sid = sid.jump(idx);
			}
			if self.sid.iter_no < sid.total_iter {
				sid.iter_no = self.sid.iter_no;
			}
		}

		self.config = config;
		self.sid = sid;
		Ok(())
	}

	async fn handle_commands<const R: bool>(&self) -> Result<Event, Error> {
		let mut buffer = Vec::new();
		loop {
			let mut stream = self.listener.listen().await?;
			buffer.clear();
			let msg = stream.read(&mut buffer).await?;
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
				Command::Finish(_) => return Ok(Event::Finished),
				Command::Jump(JumpArgs { id }) => if let Some(idx) = self.config.idmap.get(&id) {
					return Ok(Event::Jump(*idx));
				}
				Command::Reload(_) => return Ok(Event::Command(command)),
				Command::Fetch(FetchArgs { format }) =>
					return Ok(Event::Fetch(format, stream)),
				Command::Listen(ListenArgs { overrid }) => return Ok(Event::Listen(overrid, stream)),
				_ => {}
			}
		}
	}

	async fn handle_fetch_resumed(&self, format: String, mut stream: Stream, dest: Instant) -> Result<(), Error> {
		let tokens = Token::parse(&format);
		let session = &self.config.sessions[self.sid.curr()];
		let remaining = dest - Instant::now();
		let displayed = session.display_with_format::<true>(remaining, &tokens);
		stream.write(format!("{}", displayed).as_bytes()).await?;
		Ok(())
	}

	async fn handle_fetch_paused(&self, format: String, mut stream: Stream, duration: Duration) -> Result<(), Error> {
		let tokens = Token::parse(&format);
		let session = &self.config.sessions[self.sid.curr()];
		let displayed = session.display_with_format::<false>(duration, &tokens);
		stream.write(format!("{}", displayed).as_bytes()).await?;
		Ok(())
	}

	fn initial_state(&self) -> State {
		if self.config.iterations != Some(0) && !self.config.sessions.is_empty() {
			self.new_state()
		} else {
			State::Finished
		}
	}

	fn initial_jump(&mut self, idx: usize) -> State {
		if self.config.iterations != Some(0) {
			self.jump_session(idx)
		} else {
			State::Finished
		}
	}

	fn curr_session(&self) -> &Session {
		&self.config.sessions[self.sid.curr()]
	}

	fn next_session(&mut self) -> State {
		self.sid = self.sid.next();
		self.new_state()
	}

	fn prev_session(&mut self) -> State {
		self.sid = self.sid.prev();
		self.new_state()
	}

	fn jump_session(&mut self, idx: usize) -> State {
		self.sid = self.sid.jump(idx);
		self.new_state()
	}

	fn new_state(&self) -> State {
		let session = self.curr_session();
		if session.autostart {
			let start = Instant::now();
			State::Resumed(start, start + session.duration)
		} else {
			State::Paused(session.duration)
		}
	}
}

enum State {
	Paused(Duration),
	Resumed(Instant, Instant),
	Finished,
}

#[cfg(test)]
mod tests {
	use crate::{app::App, Args};

	#[test]
	fn indicate_missing_config_file() {
		let result = App::new(Args {
			config: "~/.config/uair/no_uair.toml".into(),
			socket: "/tmp/uair.sock".into(),
		});
		assert_eq!(
			result.err().unwrap().to_string(),
			"IO Error: Could not load config file \"~/.config/uair/no_uair.toml\"",
		);
	}
}
