use std::fmt::{self, Display, Formatter};
use std::io;
use std::process;
use std::time::Duration;
use humantime::format_duration;
use nom8::{IResult, Parser};
use nom8::branch::alt;
use nom8::bytes::{any, one_of, take_until};
use nom8::combinator::{opt, peek, rest};
use nom8::multi::many0;
use nom8::sequence::preceded;

pub struct Session {
	pub name: String,
	pub duration: Duration,
	pub command: String,
	pub format: Vec<Token>,
	pub time_format: Vec<TimeFormatToken>,
	pub autostart: bool,
	pub paused_state_text: String,
	pub resumed_state_text: String,
}

impl Session {
	pub fn display<const R: bool>(&self, time: Duration) -> DisplayableSession<'_, '_, '_, R> {
		DisplayableSession {
			session: self,
			time: DisplayableTime { time, format: &self.time_format },
			format: &self.format
		}
	}

	pub fn display_with_format<'session, 'token, const R: bool>(
		&'session self,
		time: Duration,
		format: &'token [Token]
	) -> DisplayableSession<'session, 'token, '_, R> {
		DisplayableSession {
			session: self,
			time: DisplayableTime { time, format: &self.time_format },
			format
		}
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

pub struct DisplayableSession<'session, 'token, 'tftoken, const R: bool> {
	session: &'session Session,
	time: DisplayableTime<'tftoken>,
	format: &'token[Token],
}

impl<'session, 'token, 'tftoken, const R: bool> Display for DisplayableSession<'session, 'token, 'tftoken, R> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		for token in self.format {
			match token {
				Token::Name => write!(f, "{}", self.session.name)?,
				Token::Percent => write!(f, "{}", (
					self.time.time.as_secs_f32() * 100.0 / self.session.duration.as_secs_f32()
				) as u8)?,
				Token::Time => write!(f, "{}", self.time)?,
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

struct DisplayableTime<'tftoken> {
	time: Duration,
	format: &'tftoken[TimeFormatToken],
}

impl<'tftoken> Display for DisplayableTime<'tftoken> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		let secs = self.time.as_secs();
		let years = secs / 31_557_600;
		let ydays = secs % 31_557_600;
		let months = ydays / 2_630_016;
		let mdays = ydays % 2_630_016;
		let days = mdays / 86400;
		let day_secs = mdays % 86400;
		let hours = day_secs / 3600;
		let minutes = day_secs % 3600 / 60;
		let seconds = day_secs % 60;

		let mut skip = false;
		let mut plural = "";
		for token in self.format {
			match token {
				TimeFormatToken::Numeric(n, pad, s) => {
					let val = match n {
						Numeric::Year => years,
						Numeric::Month => months,
						Numeric::Day => days,
						Numeric::Hour => hours,
						Numeric::Minute => minutes,
						Numeric::Second => seconds,
					};

					if *s && val == 0 {
						skip = true;
						continue;
					}
					skip = false;

					plural = if val > 1 { "s" } else { "" };

					match pad {
						Pad::Zero => write!(f, "{:0>2}", val)?,
						Pad::Space => write!(f, "{:>2}", val)?,
						Pad::None => write!(f, "{}", val)?,
					}
				}
				TimeFormatToken::Literal(literal) if !skip => write!(f, "{}", literal)?,
				TimeFormatToken::Plural if !skip => write!(f, "{}", plural)?,
				_ => {},
			}
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
pub enum TimeFormatToken {
	Numeric(Numeric, Pad, bool),
	Literal(String),
	Plural,
}

impl TimeFormatToken {
	pub fn parse(format: &str) -> Vec<TimeFormatToken> {
		let res: IResult<&str, Vec<TimeFormatToken>> = many0(alt((
			preceded("%", (opt(one_of("*")), opt(one_of("-_0")), opt(any)).map(Self::identify)),
			take_until("%").map(|s: &str| TimeFormatToken::Literal(s.into())),
			(peek(any), rest).map(|(_, s): (char, &str)| TimeFormatToken::Literal(s.into())),
		)))(format);
		res.unwrap().1
	}

	fn identify((star, flag, spec): (Option<char>, Option<char>, Option<char>)) -> TimeFormatToken {
		let skip = star.is_some();
		let pad = match flag {
			Some('-') => Pad::None,
			Some('_') => Pad::Space,
			Some('0') | None => Pad::Zero,
			_ => unreachable!(),
		};
		match spec {
			Some('Y') => TimeFormatToken::Numeric(Numeric::Year, pad, skip),
			Some('B') => TimeFormatToken::Numeric(Numeric::Month, pad, skip),
			Some('D') => TimeFormatToken::Numeric(Numeric::Day, pad, skip),
			Some('H') => TimeFormatToken::Numeric(Numeric::Hour, pad, skip),
			Some('M') => TimeFormatToken::Numeric(Numeric::Minute, pad, skip),
			Some('S') => TimeFormatToken::Numeric(Numeric::Second, pad, skip),
			Some('P') => TimeFormatToken::Plural,
			_ => {
				let mut l = "%".to_string();
				if let Some(s) = star { l.push(s) };
				if let Some(f) = flag { l.push(f) };
				if let Some(c) = spec { l.push(c) };
				TimeFormatToken::Literal(l)
			},
		}
	}
}

#[cfg_attr(test, derive(PartialEq, Debug))]
pub enum Numeric {
	Year,
	Month,
	Day,
	Hour,
	Minute,
	Second,
}

#[cfg_attr(test, derive(PartialEq, Debug))]
pub enum Pad {
	Zero,
	Space,
	None,
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
	use super::{Color, Numeric, Pad, TimeFormatToken, Token};

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

	#[test]
	fn parse_time_format() {
		assert_eq!(&TimeFormatToken::parse("%H:%M:%S"), &[
			TimeFormatToken::Numeric(Numeric::Hour, Pad::Zero, false),
			TimeFormatToken::Literal(":".into()),
			TimeFormatToken::Numeric(Numeric::Minute, Pad::Zero, false),
			TimeFormatToken::Literal(":".into()),
			TimeFormatToken::Numeric(Numeric::Second, Pad::Zero, false),
		]);
		assert_eq!(&TimeFormatToken::parse("%L:%M:%S"), &[
			TimeFormatToken::Literal("%L".into()),
			TimeFormatToken::Literal(":".into()),
			TimeFormatToken::Numeric(Numeric::Minute, Pad::Zero, false),
			TimeFormatToken::Literal(":".into()),
			TimeFormatToken::Numeric(Numeric::Second, Pad::Zero, false),
		]);
		assert_eq!(&TimeFormatToken::parse("%H:%M:%"), &[
			TimeFormatToken::Numeric(Numeric::Hour, Pad::Zero, false),
			TimeFormatToken::Literal(":".into()),
			TimeFormatToken::Numeric(Numeric::Minute, Pad::Zero, false),
			TimeFormatToken::Literal(":".into()),
			TimeFormatToken::Literal("%".into()),
		]);
		assert_eq!(&TimeFormatToken::parse("%_H:%-M:%S"), &[
			TimeFormatToken::Numeric(Numeric::Hour, Pad::Space, false),
			TimeFormatToken::Literal(":".into()),
			TimeFormatToken::Numeric(Numeric::Minute, Pad::None, false),
			TimeFormatToken::Literal(":".into()),
			TimeFormatToken::Numeric(Numeric::Second, Pad::Zero, false),
		]);
		assert_eq!(&TimeFormatToken::parse("%H:%*-M:%S"), &[
			TimeFormatToken::Numeric(Numeric::Hour, Pad::Zero, false),
			TimeFormatToken::Literal(":".into()),
			TimeFormatToken::Numeric(Numeric::Minute, Pad::None, true),
			TimeFormatToken::Literal(":".into()),
			TimeFormatToken::Numeric(Numeric::Second, Pad::Zero, false),
		]);
		assert_eq!(&TimeFormatToken::parse("%*-Hh %*-Mm %-Ss"), &[
			TimeFormatToken::Numeric(Numeric::Hour, Pad::None, true),
			TimeFormatToken::Literal("h ".into()),
			TimeFormatToken::Numeric(Numeric::Minute, Pad::None, true),
			TimeFormatToken::Literal("m ".into()),
			TimeFormatToken::Numeric(Numeric::Second, Pad::None, false),
			TimeFormatToken::Literal("s".into()),
		]);
	}
}
