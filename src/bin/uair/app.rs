use std::process;
use std::time::Duration;
use futures_lite::FutureExt;
use uair::Command;
use super::server::Listener;
use super::timer::UairTimer;
use super::config::UairConfig;

pub async fn run(config: UairConfig) -> anyhow::Result<()> {
	let listener = Listener::new("/tmp/uair.sock")?;
	let mut i = 0;
	while i < config.nb_sessions() {
		let mut timer = UairTimer::new(
			config.duration(i),
			Duration::from_secs(1),
			config.before(i),
			config.after(i)
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

		process::Command::new("sh")
			.arg("-c")
			.arg(config.command(i))
			.spawn()?;

		i += 1;
	}

	Ok(())
}

pub enum Event {
	Command(Command),
	Finished,
}
