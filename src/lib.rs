use std::env;
use argh::FromArgs;
use serde::{Serialize, Deserialize};

#[derive(FromArgs, Serialize, Deserialize)]
#[argh(subcommand)]
pub enum Command {
	Pause(PauseArgs),
	Resume(ResumeArgs),
	Toggle(ToggleArgs),
	Next(NextArgs),
	Prev(PrevArgs),
}

#[derive(FromArgs, Serialize, Deserialize)]
/// Pause the timer.
#[argh(subcommand, name = "pause")]
pub struct PauseArgs {}

#[derive(FromArgs, Serialize, Deserialize)]
/// Resume the timer.
#[argh(subcommand, name = "resume")]
pub struct ResumeArgs {}

#[derive(FromArgs, Serialize, Deserialize)]
/// Toggle the state of the timer.
#[argh(subcommand, name = "toggle")]
pub struct ToggleArgs {}

#[derive(FromArgs, Serialize, Deserialize)]
/// Jump to the next session.
#[argh(subcommand, name = "next")]
pub struct NextArgs {}

#[derive(FromArgs, Serialize, Deserialize)]
/// Jump to the previous session.
#[argh(subcommand, name = "prev")]
pub struct PrevArgs {}

pub fn get_socket_path() -> String {
	if let Ok(xdg_runtime_dir) = env::var("XDG_RUNTIME_DIR") {
		xdg_runtime_dir + "/uair.sock"
	} else if let Ok(tmp_dir) = env::var("TMPDIR") {
		tmp_dir + "/uair.sock"
	} else {
		"/tmp/uair.sock".into()
	}
}
