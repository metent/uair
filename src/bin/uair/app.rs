use std::io::{self, Write};
use std::process;
use std::time::Duration;
use futures_lite::FutureExt;
use super::Args;
use super::server::Listener;
use super::timer::UairTimer;
use super::config::UairConfig;

pub async fn run(args: Args, config: UairConfig) -> anyhow::Result<()> {
	let mut stdout = io::stdout();
	write!(stdout, "{}", config.startup_text)?;
	stdout.flush()?;

	let listener = Listener::new(&args.socket)?;
	if config.pause_at_start { listener.wait_while_stopped(0).await?; }

	let mut i = 0;
	while i < config.nb_sessions() {
		let mut timer = UairTimer::new(
			config.duration(i),
			Duration::from_secs(1),
			config.before(i),
			config.after(i)
		);

		if !config.autostart(i) { listener.wait_while_stopped(i).await?; }
		loop {
			match timer.start().or(listener.wait_while_running(i)).await? {
				Event::Pause => {
					timer.update_duration();
					listener.wait_while_stopped(i).await?;
				}
				Event::Finished => {
					let command = config.command(i);
					if !command.is_empty() {
						process::Command::new("sh")
							.arg("-c")
							.arg(command)
							.spawn()?;
					}
					i += 1;
					if config.loop_on_end { i %= config.nb_sessions(); }
					break;
				}
				Event::Next => {
					i += 1;
					if config.loop_on_end { i %= config.nb_sessions(); }
					break;
				}
				Event::Prev => {
					if i != 0 {
						i -= 1;
					} else if config.loop_on_end {
						i = config.nb_sessions() - 1;
					}
				}
				_ => {}
			}
		}

	}

	Ok(())
}

pub enum Event {
	Pause,
	Resume,
	Finished,
	Next,
	Prev,
}
