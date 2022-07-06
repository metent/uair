mod server;
mod timer;

use std::time::Duration;
use futures_lite::FutureExt;
use server::{Event, Listener};
use timer::UairTimer;

fn main() {
	smol::block_on(amain());
}

async fn amain() {
	let listener = Listener::new("/tmp/uair.sock");
	let mut timer = UairTimer::new(Duration::from_millis(50700), Duration::from_secs(1));

	loop {
		match timer.start().or(listener.listen()).await {
			Event::Pause => {
				timer.update_duration();
				loop {
					if let Event::Start = listener.listen().await { break; }
				}
			}
			Event::Stop => break,
			Event::Start => {}
		}
	}
}
