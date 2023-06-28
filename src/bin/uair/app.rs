use std::io::{self, Write, Error as IoError, ErrorKind};
use std::fs::{self, File};
use std::time::{Duration, Instant};
use uair::{Command, FetchArgs, ListenArgs, PauseArgs, ResumeArgs, JumpArgs};
use log::{LevelFilter, error};
use futures_lite::FutureExt;
use simplelog::{ColorChoice, Config as LogConfig, TermLogger, TerminalMode, WriteLogger};
use crate::{Args, Error};
use crate::config::{Config, ConfigBuilder};
use crate::socket::{Listener, Stream};
use crate::session::{Session, SessionId, Token};
use crate::timer::{State, UairTimer};

pub struct App {
	data: AppData,
	timer: UairTimer,
}

impl App {
	pub fn new(args: Args) -> Result<Self, Error> {
		if args.log == "-" {
			TermLogger::init(
				LevelFilter::Info,
				LogConfig::default(),
				TerminalMode::Stderr,
				ColorChoice::Auto
			)?;
		} else {
			WriteLogger::init(
				LevelFilter::Info,
				LogConfig::default(),
				File::create(&args.log)?
			)?;
		}
		let timer = UairTimer::new(Duration::from_secs(1), args.quiet);
		let data = AppData::new(args)?;
		Ok(App { data, timer })
	}

	pub async fn run(mut self) -> Result<(), Error> {
		let mut stdout = io::stdout();
		write!(stdout, "{}", self.data.config.startup_text)?;
		stdout.flush()?;

		loop {
			match match self.timer.state {
				State::PreInit => self.start_up().await,
				State::Paused(duration) => self.pause_session(duration).await,
				State::Resumed(start, dest) => self.run_session(start, dest).await,
				State::Finished => break,
			} {
				Err(Error::ConfError(err)) => error!("{}", err),
				Err(Error::DeserError(err)) => error!("{}", err),
				Err(err) => return Err(err),
				_ => {},
			}
		}
		Ok(())
	}

	async fn start_up(&mut self) -> Result<(), Error> {
		if !self.data.config.pause_at_start {
			self.timer.state = self.data.initial_state();
			return Ok(());
		}

		match self.data.handle_commands::<false>().await? {
			Event::Finished | Event::Command(Command::Resume(_) | Command::Next(_)) => {
				self.timer.state = self.data.initial_state();
			}
			Event::Command(Command::Prev(_)) => {},
			Event::Jump(idx) => {
				self.timer.state = self.data.initial_jump(idx);
			}
			Event::Command(Command::Reload(_)) => self.data.read_conf::<true>()?,
			Event::Fetch(format, stream) =>
				self.data.handle_fetch_paused(format, stream, Duration::ZERO).await?,
			Event::Listen(overrid, stream) =>
				self.timer.writer.add_stream(stream.into_blocking(), overrid),
			_ => unreachable!(),
		}

		Ok(())
	}

	async fn run_session(&mut self, start: Instant, dest: Instant) -> Result<(), Error> {
		match self.timer.start(self.data.curr_session(), start, dest)
			.or(self.data.handle_commands::<true>()).await? {
			Event::Finished => {
				let res = self.data.curr_session().run_command();
				self.timer.state = if self.data.sid.is_last() {
					State::Finished
				} else {
					self.data.next_session()
				};
				res?;
			}
			Event::Command(Command::Pause(_)) =>
				self.timer.state = State::Paused(dest - Instant::now()),
			Event::Command(Command::Next(_)) =>
				self.timer.state = self.data.next_session(),
			Event::Command(Command::Prev(_)) =>
				self.timer.state = self.data.prev_session(),
			Event::Jump(idx) =>
				self.timer.state = self.data.jump_session(idx),
			Event::Command(Command::Reload(_)) => self.data.read_conf::<true>()?,
			Event::Fetch(format, stream) =>
				self.data.handle_fetch_resumed(format, stream, dest).await?,
			Event::Listen(overrid, stream) =>
				self.timer.writer.add_stream(stream.into_blocking(), overrid),
			_ => unreachable!(),
		}
		Ok(())
	}

	async fn pause_session(&mut self, duration: Duration) -> Result<(), Error> {
		const DELTA: Duration = Duration::from_nanos(1_000_000_000 - 1);

		self.timer.writer.write::<false>(self.data.curr_session(), duration + DELTA)?;

		match self.data.handle_commands::<false>().await? {
			Event::Finished => {
				let res = self.data.curr_session().run_command();
				self.timer.state = if self.data.sid.is_last() {
					State::Finished
				} else {
					self.data.next_session()
				};
				res?;
			}
			Event::Command(Command::Resume(_)) => {
				let start = Instant::now();
				self.timer.state = State::Resumed(start, start + duration);
				self.timer.writer.write::<true>(self.data.curr_session(), duration + DELTA)?;
			}
			Event::Command(Command::Next(_)) =>
				self.timer.state = self.data.next_session(),
			Event::Command(Command::Prev(_)) =>
				self.timer.state = self.data.prev_session(),
			Event::Jump(idx) => self.timer.state = self.data.jump_session(idx),
			Event::Command(Command::Reload(_)) => self.data.read_conf::<true>()?,
			Event::Fetch(format, stream) =>
				self.data.handle_fetch_paused(format, stream, duration + DELTA).await?,
			Event::Listen(overrid, stream) =>
				self.timer.writer.add_stream(stream.into_blocking(), overrid),
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

#[cfg(test)]
mod tests {
	use crate::{app::App, Args};

	#[test]
	fn indicate_missing_config_file() {
		let result = App::new(Args {
			config: "~/.config/uair/no_uair.toml".into(),
			socket: "/tmp/uair.sock".into(),
			log: "-".into(),
			quiet: false,
			version: false,
		});
		assert_eq!(
			result.err().unwrap().to_string(),
			"IO Error: Could not load config file \"~/.config/uair/no_uair.toml\"",
		);
	}
}
