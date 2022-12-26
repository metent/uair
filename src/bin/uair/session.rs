use std::io::{self, Write};
use std::time::Duration;
use humantime::format_duration;

pub struct Session {
	pub name: String,
	pub duration: Duration,
	pub command: String,
	pub format: Vec<Token>,
	pub autostart: bool,
}

impl Session {
	pub fn display(&self, time: Duration) -> io::Result<()> {
		let mut stdout = io::stdout();
		for token in &self.format {
			match token {
				Token::Name => write!(stdout, "{}", self.name)?,
				Token::Percent => write!(stdout, "{}", (
					time.as_secs_f32() * 100.0 / self.duration.as_secs_f32()
				) as u8)?,
				Token::Time => write!(stdout, "{}", format_duration(time))?,
				Token::Total => write!(stdout, "{}", format_duration(self.duration))?,
				Token::Color(Color::Black) => write!(stdout, "{}", "\x1b[0;30m")?,
				Token::Color(Color::Red) => write!(stdout, "{}", "\x1b[0;31m")?,
				Token::Color(Color::Green) => write!(stdout, "{}", "\x1b[0;32m")?,
				Token::Color(Color::Yellow) => write!(stdout, "{}", "\x1b[0;33m")?,
				Token::Color(Color::Blue) => write!(stdout, "{}", "\x1b[0;34m")?,
				Token::Color(Color::Purple) => write!(stdout, "{}", "\x1b[0;35m")?,
				Token::Color(Color::Cyan) => write!(stdout, "{}", "\x1b[0;36m")?,
				Token::Color(Color::White) => write!(stdout, "{}", "\x1b[0;37m")?,
				Token::Color(Color::End) => write!(stdout, "{}", "\x1b[0m")?,
				Token::Literal(literal) => write!(stdout, "{}", literal)?,
			};
		}
		stdout.flush()?;
		Ok(())
	}
}

#[cfg_attr(test, derive(PartialEq, Debug))]
pub enum Token {
	Name,
	Percent,
	Time,
	Total,
	Color(Color),
	Literal(String),
}

#[cfg_attr(test, derive(PartialEq, Debug))]
pub enum Color {
	Black,
	Red,
	Green,
	Yellow,
	Blue,
	Purple,
	Cyan,
	White,
	End,
}

pub struct SessionId {
	index: usize,
	len: usize,
	iter_no: u64,
	total_iter: u64,
	infinite: bool,
}

impl SessionId {
	pub fn new(sessions: &[Session], iterations: Option<u64>) -> Self {
		SessionId {
			index: 0,
			len: sessions.len(),
			iter_no: 0,
			total_iter: iterations.unwrap_or(0),
			infinite: iterations.is_none(),
		}
	}

	pub fn curr(&self) -> usize {
		self.index
	}

	pub fn next(&mut self) {
		if self.index < self.len - 1 {
			self.index += 1;
		} else if self.infinite {
			self.index = 0;
		} else if self.iter_no < self.total_iter - 1 {
			self.index = 0;
			self.iter_no += 1;
		}
	}

	pub fn prev(&mut self) {
		if self.index > 0 {
			self.index -= 1;
		} else if self.infinite {
			self.index = self.len - 1
		} else if self.iter_no > 0 {
			self.index = self.len - 1;
			self.iter_no -= 1;
		}
	}

	pub fn is_last(&self) -> bool {
		self.index == self.len - 1 && !self.infinite && self.iter_no == self.total_iter - 1
	}

	pub fn is_first(&self) -> bool {
		self.index == 0 && !self.infinite && self.iter_no == 0
	}
}
