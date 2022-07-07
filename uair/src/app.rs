use std::time::Duration;
use futures_lite::FutureExt;
use common::Command;
use super::server::Listener;
use super::timer::UairTimer;
use super::config::UairConfig;

pub async fn run(config: UairConfig) -> anyhow::Result<()> {
	let listener = Listener::new("/tmp/uair.sock")?;
	for session in config.sessions {
		let mut timer = UairTimer::new(
			session.duration,
			Duration::from_secs(1),
			session.before,
			session.after
		);

		listener.wait_for_resume().await?;
		loop {
			match timer.start().or(listener.listen()).await? {
				Event::Command(Command::Pause | Command::Toggle) => {
					timer.update_duration();
					listener.wait_for_resume().await?;
				}
				Event::Finished => break,
				_ => {}
			}
		}
	}

	Ok(())
}

pub enum Event {
	Command(Command),
	Finished,
}
