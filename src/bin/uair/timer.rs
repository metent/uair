use crate::app::Event;
use crate::session::Session;
use crate::socket::BlockingStream;
use crate::Error;
use async_io::Timer;
use std::fmt::Write as _;
use std::io::{self, Stdout, Write};
use std::time::{Duration, Instant};

pub struct UairTimer {
	interval: Duration,
	pub writer: Writer,
	pub state: State,
}

impl UairTimer {
	pub fn new(interval: Duration, quiet: bool) -> Self {
		UairTimer {
			interval,
			writer: Writer::new(quiet),
			state: State::PreInit,
		}
	}

	pub async fn start(
		&mut self,
		session: &Session,
		start: Instant,
		dest: Instant,
	) -> Result<Event, Error> {
		let _guard = StateGuard(&mut self.state);

		let duration = dest - start;
		let first_interval = Duration::from_nanos(duration.subsec_nanos().into());
		let mut end = start + first_interval;

		while end <= dest {
			Timer::at(end).await;
			self.writer.write::<true>(session, dest - end)?;
			end += self.interval;
		}

		Ok(Event::Finished)
	}
}

pub struct Writer {
	streams: Vec<(BlockingStream, Option<String>)>,
	stdout: Option<Stdout>,
	buf: String,
}

impl Writer {
	fn new(quiet: bool) -> Self {
		Writer {
			streams: Vec::new(),
			stdout: (!quiet).then(io::stdout),
			buf: "".into(),
		}
	}

	pub fn write<const R: bool>(
		&mut self,
		session: &Session,
		duration: Duration,
	) -> Result<(), Error> {
		if let Some(stdout) = &mut self.stdout {
			_ = write!(self.buf, "{}", session.display::<R>(duration, None));
			if write!(stdout, "{}", self.buf)
				.and_then(|_| stdout.flush())
				.is_err()
			{
				self.stdout = None;
			}
			self.buf.clear();
		}
		self.streams.retain_mut(|(stream, overrid)| {
			let overrid = overrid.as_ref().and_then(|o| session.overrides.get(o));
			_ = write!(self.buf, "{}\0", session.display::<R>(duration, overrid));
			let res = stream.write(self.buf.as_bytes()).is_ok();
			self.buf.clear();
			res
		});
		Ok(())
	}

	pub fn add_stream(&mut self, stream: BlockingStream, overrid: Option<String>) {
		self.streams.push((stream, overrid));
	}
}

pub enum State {
	PreInit,
	Paused(Duration),
	Resumed(Instant, Instant),
	Finished,
}

struct StateGuard<'s>(&'s mut State);

impl Drop for StateGuard<'_> {
	fn drop(&mut self) {
		if let State::Resumed(_, dest) = self.0 {
			*self.0 = State::Resumed(Instant::now(), *dest);
		}
	}
}
