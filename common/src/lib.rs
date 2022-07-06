use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum Command {
	Pause,
	Resume,
	Toggle,
}
