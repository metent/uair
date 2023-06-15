use std::io::{self, Write};
use std::time::{Duration, Instant};
use async_io::Timer;
use crate::Error;
use crate::app::Event;
use crate::session::Session;
use crate::socket::BlockingStream;

pub struct UairTimer {
	interval: Duration,
	streams: Vec<BlockingStream>,
}

impl UairTimer {
	pub fn new(interval: Duration) -> Self {
		UairTimer { interval, streams: Vec::new() }
	}

	pub async fn start(&mut self, session: &Session, start: Instant, dest: Instant) -> Result<Event, Error> {
		let mut stdout = io::stdout();

		let duration = dest - start;
		let first_interval = Duration::from_nanos(duration.subsec_nanos().into());
		let mut end = start + first_interval;

		while end <= dest {
			Timer::at(end).await;
			let time = session.display::<true>(dest - end).to_string() + "\0";
			write!(stdout, "{}", time)?;
			self.streams.retain_mut(|stream| stream.write(time.as_bytes()).is_ok());
			stdout.flush()?;
			end += self.interval;
		}

		Ok(Event::Finished)
	}

	pub fn add_stream(&mut self, stream: BlockingStream) {
		self.streams.push(stream);
	}
}
