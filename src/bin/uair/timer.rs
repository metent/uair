use std::io::{self, Stdout, Write};
use std::fmt::Write as _;
use std::time::{Duration, Instant};
use async_io::Timer;
use crate::Error;
use crate::app::Event;
use crate::session::{Session, Overridables};
use crate::socket::BlockingStream;

pub struct UairTimer {
	interval: Duration,
	streams: Vec<(BlockingStream, Option<Overridables>)>,
	stdout: Stdout,
	buf: String,
}

impl UairTimer {
	pub fn new(interval: Duration) -> Self {
		UairTimer { stdout: io::stdout(), interval, streams: Vec::new(), buf: "".into() }
	}

	pub async fn start(&mut self, session: &Session, start: Instant, dest: Instant) -> Result<Event, Error> {
		let duration = dest - start;
		let first_interval = Duration::from_nanos(duration.subsec_nanos().into());
		let mut end = start + first_interval;

		while end <= dest {
			Timer::at(end).await;
			self.write::<true>(session, dest - end)?;
			end += self.interval;
		}

		Ok(Event::Finished)
	}

	pub fn write<const R: bool>(&mut self, session: &Session, duration: Duration) -> Result<(), Error> {
		write!(self.buf, "{}", session.display::<R>(duration, None))?;
		write!(self.stdout, "{}", self.buf)?;
		self.stdout.flush()?;
		self.buf.clear();
		self.streams.retain_mut(|(stream, overrid)| {
			if write!(self.buf, "{}\0", session.display::<R>(duration, overrid.as_ref())).is_err() {
				self.buf += "Formatting Error";
			}
			let res = stream.write(self.buf.as_bytes()).is_ok();
			self.buf.clear();
			res
		});
		Ok(())
	}

	pub fn add_stream(&mut self, stream: BlockingStream, overrid: Option<Overridables>) {
		self.streams.push((stream, overrid));
	}
}
