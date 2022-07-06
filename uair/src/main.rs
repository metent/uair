mod server;
mod timer;

use std::time::Duration;
use futures_lite::FutureExt;
use server::Listener;
use timer::UairTimer;
use common::Command;

fn main() -> anyhow::Result<()> {
	smol::block_on(amain())?;
	Ok(())
}

async fn amain() -> anyhow::Result<()> {
	let listener = Listener::new("/tmp/uair.sock")?;
	let mut timer = UairTimer::new(Duration::from_millis(50700), Duration::from_secs(1));

	loop {
		match timer.start().or(listener.listen()).await? {
			Event::Command(Command::Pause) => {
				timer.update_duration();
				loop {
					if let Event::Command(Command::Resume) = listener.listen().await? { break; }
				}
			}
			Event::Finished => break,
			_ => {}
		}
	}

	Ok(())
}

pub enum Event {
	Command(Command),
	Finished,
}
