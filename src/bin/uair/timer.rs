use std::io::{self, Write};
use std::time::{Duration, Instant};
use async_io::Timer;
use crate::Error;
use crate::app::Event;
use crate::session::Session;

pub struct UairTimer {
	pub duration: Duration,
	interval: Duration,
}

impl UairTimer {
	pub fn new(duration: Duration, interval: Duration) -> Self {
		UairTimer { duration, interval }
	}

	pub async fn start(&self, session: &Session, started: Instant) -> Result<Event, Error> {
		let mut stdout = io::stdout();

		let first_interval = Duration::from_nanos(self.duration.subsec_nanos().into());

		let mut end = started + first_interval;
		let dest = started + self.duration;

		while end <= dest {
			Timer::at(end).await;
			write!(stdout, "{}", session.display::<true>(dest - end))?;
			stdout.flush()?;
			end += self.interval;
		}

		Ok(Event::Finished)
	}
}
