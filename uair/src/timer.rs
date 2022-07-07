use std::io::{self, Write};
use std::time::{Duration, Instant};
use smol::Timer;
use super::app::Event;

pub struct UairTimer {
	duration: Duration,
	interval: Duration,
	started: Instant,
	before: String,
	after: String,
}

impl UairTimer {
	pub fn new(duration: Duration, interval: Duration, before: String, after: String) -> Self {
		UairTimer { duration, interval, started: Instant::now(), before, after }
	}

	pub async fn start(&mut self) -> anyhow::Result<Event> {
		let mut stdout = io::stdout();
		let first_interval = Duration::from_nanos(self.duration.subsec_nanos().into());

		self.started = Instant::now();
		let mut end = self.started + first_interval;
		let dest = self.started + self.duration;

		while end <= dest {
			Timer::at(end).await;
			write!(
				stdout, "{}{}{}",
				self.before, humantime::format_duration(dest - end), self.after
			)?;
			stdout.flush()?;
			end += self.interval;
		}

		Ok(Event::Finished)
	}

	pub fn update_duration(&mut self) {
		self.duration -= Instant::now() - self.started;
	}
}
