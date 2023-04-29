use std::io::{self, Write};
use std::time::{Duration, Instant};
use async_io::Timer;
use crate::Error;
use crate::app::Event;
use crate::session::Session;

pub struct UairTimer {
	interval: Duration,
}

impl UairTimer {
	pub fn new(interval: Duration) -> Self {
		UairTimer { interval }
	}

	pub async fn start(&self, session: &Session, start: Instant, dest: Instant) -> Result<Event, Error> {
		let mut stdout = io::stdout();

		let duration = dest - start;
		let first_interval = Duration::from_nanos(duration.subsec_nanos().into());
		let mut end = start + first_interval;

		while end <= dest {
			Timer::at(end).await;
			write!(stdout, "{}", session.display::<true>(dest - end))?;
			stdout.flush()?;
			end += self.interval;
		}

		Ok(Event::Finished)
	}
}
