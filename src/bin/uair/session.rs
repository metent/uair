use std::fmt::{self, Display, Formatter};
use std::io;
use std::process;
use std::time::Duration;
use humantime::format_duration;

pub struct Session {
	pub name: String,
	pub duration: Duration,
	pub command: String,
	pub format: Vec<Token>,
	pub autostart: bool,
	pub paused_state_text: String,
	pub resumed_state_text: String,
}

impl Session {
	pub fn display<const R: bool>(&self, time: Duration) -> DisplayableSession<'_, '_, R> {
		DisplayableSession { session: self, time, format: &self.format }
	}

	pub fn display_with_format<'session, 'token, const R: bool>(
		&'session self,
		time: Duration,
		format: &'token [Token]
	) -> DisplayableSession<'session, 'token, R> {
		DisplayableSession { session: self, time, format }
	}

	pub fn run_command(&self) -> io::Result<()> {
		if !self.command.is_empty() {
			let duration = humantime::format_duration(self.duration).to_string();
			process::Command::new("sh")
				.env("name", &self.name)
				.env("duration", duration)
				.arg("-c")
				.arg(&self.command)
				.spawn()?;
		}
		Ok(())
	}
}

pub struct DisplayableSession<'session, 'token,  const R: bool> {
	session: &'session Session,
	time: Duration,
	format: &'token[Token],
}

impl<'session, 'token, const R: bool> Display for DisplayableSession<'session, 'token, R> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		for token in self.format {
			match token {
				Token::Name => write!(f, "{}", self.session.name)?,
				Token::Percent => write!(f, "{}", (
					self.time.as_secs_f32() * 100.0 / self.session.duration.as_secs_f32()
				) as u8)?,
				Token::Time => write!(f, "{}",
					format_duration(Duration::from_secs(self.time.as_secs())))?,
				Token::Total => write!(f, "{}", format_duration(self.session.duration))?,
				Token::State => write!(f, "{}", if R {
					&self.session.resumed_state_text
				} else {
					&self.session.paused_state_text
				})?,
				Token::Color(Color::Black) => write!(f, "{}", "\x1b[0;30m")?,
				Token::Color(Color::Red) => write!(f, "{}", "\x1b[0;31m")?,
				Token::Color(Color::Green) => write!(f, "{}", "\x1b[0;32m")?,
				Token::Color(Color::Yellow) => write!(f, "{}", "\x1b[0;33m")?,
				Token::Color(Color::Blue) => write!(f, "{}", "\x1b[0;34m")?,
				Token::Color(Color::Purple) => write!(f, "{}", "\x1b[0;35m")?,
				Token::Color(Color::Cyan) => write!(f, "{}", "\x1b[0;36m")?,
				Token::Color(Color::White) => write!(f, "{}", "\x1b[0;37m")?,
				Token::Color(Color::End) => write!(f, "{}", "\x1b[0m")?,
				Token::Literal(literal) => write!(f, "{}", literal)?,
			};
		}
		Ok(())
	}
}

#[cfg_attr(test, derive(PartialEq, Debug))]
pub enum Token {
	Name,
	Percent,
	Time,
	Total,
	State,
	Color(Color),
	Literal(String),
}

impl Token {
	pub fn parse(format: &str) -> Vec<Token> {
		let mut tokens = Vec::new();
		let mut k = 0;
		let mut open = None;

		for (i, c) in format.char_indices() {
			match c {
				'{' => open = Some(i),
				'}' => if let Some(j) = open {
					if let Ok(token) = (&format[j..=i]).parse() {
						if k != j { tokens.push(Token::Literal(format[k..j].into())) };
						tokens.push(token);
						k = i + 1;
					}
				}
				_ => {},
			}
		}
		if k != format.len() { tokens.push(Token::Literal(format[k..].into())) };

		tokens
	}
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

#[derive(Copy, Clone)]
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

	pub fn next(&self) -> SessionId {
		let mut next = SessionId { ..*self };
		if self.index < self.len - 1 {
			next.index += 1;
		} else if self.infinite {
			next.index = 0;
		} else if self.iter_no < self.total_iter - 1 {
			next.index = 0;
			next.iter_no += 1;
		}
		next
	}

	pub fn prev(&self) -> SessionId {
		let mut prev = SessionId { ..*self };
		if self.index > 0 {
			prev.index -= 1;
		} else if self.infinite {
			prev.index = self.len - 1
		} else if self.iter_no > 0 {
			prev.index = self.len - 1;
			prev.iter_no -= 1;
		}
		prev
	}

	pub fn is_last(&self) -> bool {
		self.index == self.len - 1 && !self.infinite && self.iter_no == self.total_iter - 1
	}

	pub fn is_first(&self) -> bool {
		self.index == 0 && !self.infinite && self.iter_no == 0
	}
}

#[cfg(test)]
mod tests {
	use super::{Color, Token};

	#[test]
	fn parse_format() {
		assert_eq!(&Token::parse("{cyan}{time}{end}\n"), &[
			Token::Color(Color::Cyan),
			Token::Time,
			Token::Color(Color::End),
			Token::Literal("\n".into()),
		]);
		assert_eq!(&Token::parse("String with {time} with some text ahead."), &[
			Token::Literal("String with ".into()),
			Token::Time,
			Token::Literal(" with some text ahead.".into())
		]);
		assert_eq!(&Token::parse("}}{}{{}{}}}{{}{{}}}"), &[
			Token::Literal("}}{}{{}{}}}{{}{{}}}".into())
		]);
		assert_eq!(&Token::parse("{time} text {time}"), &[
			Token::Time,
			Token::Literal(" text ".into()),
			Token::Time,
		]);
	}
}
