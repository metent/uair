use std::time::{Duration, Instant};
use async_io::Timer;
use super::app::Event;
use super::config::Session;

pub struct UairTimer {
	duration: Duration,
	interval: Duration,
	started: Instant,
}

impl UairTimer {
	pub fn new(duration: Duration, interval: Duration) -> Self {
		UairTimer { duration, interval, started: Instant::now() }
	}

	pub async fn start(&mut self, session: &Session) -> anyhow::Result<Event> {
		let first_interval = Duration::from_nanos(self.duration.subsec_nanos().into());

		self.started = Instant::now();
		let mut end = self.started + first_interval;
		let dest = self.started + self.duration;

		while end <= dest {
			Timer::at(end).await;
			session.display(humantime::format_duration(dest - end))?;
			end += self.interval;
		}

		Ok(Event::Finished)
	}

	pub fn update_duration(&mut self) {
		self.duration -= Instant::now() - self.started;
	}
}
