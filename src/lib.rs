use std::env;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum Command {
	Pause,
	Resume,
	Toggle,
}

pub fn get_socket_path() -> String {
	if let Ok(xdg_runtime_dir) = env::var("XDG_RUNTIME_DIR") {
		xdg_runtime_dir + "/uair.sock"
	} else if let Ok(tmp_dir) = env::var("TMPDIR") {
		tmp_dir + "/uair.sock"
	} else {
		"/tmp/uair.sock".into()
	}
}
