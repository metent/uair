use humantime::format_duration;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::process;
use std::time::Duration;
use winnow::combinator::{alt, opt, peek, preceded, repeat, rest};
use winnow::token::{any, one_of, take_until};
use winnow::{PResult, Parser};

pub struct Session {
	pub id: String,
	pub name: String,
	pub duration: Duration,
	pub command: String,
	pub format: Vec<Token>,
	pub time_format: Vec<TimeFormatToken>,
	pub autostart: bool,
	pub paused_state_text: String,
	pub resumed_state_text: String,
	pub overrides: HashMap<String, Overridables>,
}

impl Session {
	pub fn display<'s, const R: bool>(
		&'s self,
		time: Duration,
		overrid: Option<&'s Overridables>,
	) -> DisplayableSession<'s, R> {
		DisplayableSession {
			session: self,
			time: DisplayableTime {
				time,
				format: overrid
					.and_then(|o| o.time_format.as_ref())
					.unwrap_or(&self.time_format),
			},
			format: overrid
				.and_then(|o| o.format.as_ref())
				.unwrap_or(&self.format),
			pst_override: overrid.and_then(|o| o.paused_state_text.as_ref().map(|s| s.as_str())),
			rst_override: overrid.and_then(|o| o.resumed_state_text.as_ref().map(|s| s.as_str())),
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

#[derive(Clone, Default)]
pub struct Overridables {
	pub format: Option<Vec<Token>>,
	pub time_format: Option<Vec<TimeFormatToken>>,
	pub paused_state_text: Option<String>,
	pub resumed_state_text: Option<String>,
}

impl Overridables {
	pub fn new() -> Self {
		Overridables::default()
	}

	pub fn format(self, format: &str) -> Self {
		Overridables {
			format: Some(Token::parse(format)),
			..self
		}
	}
}

pub struct DisplayableSession<'s, const R: bool> {
	session: &'s Session,
	time: DisplayableTime<'s>,
	format: &'s [Token],
	pst_override: Option<&'s str>,
	rst_override: Option<&'s str>,
}

impl<'s, const R: bool> Display for DisplayableSession<'s, R> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		for token in self.format {
			match token {
				Token::Name => write!(f, "{}", self.session.name)?,
				Token::Percent => write!(
					f,
					"{}",
					(self.time.time.as_secs_f32() * 100.0 / self.session.duration.as_secs_f32())
						as u8
				)?,
				Token::Time => write!(f, "{}", self.time)?,
				Token::Total => write!(f, "{}", format_duration(self.session.duration))?,
				Token::State => write!(
					f,
					"{}",
					if R {
						self.rst_override
							.unwrap_or(&self.session.resumed_state_text)
					} else {
						self.pst_override.unwrap_or(&self.session.paused_state_text)
					}
				)?,
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

struct DisplayableTime<'s> {
	time: Duration,
	format: &'s [TimeFormatToken],
}

impl<'s> Display for DisplayableTime<'s> {
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
				_ => {}
			}
		}
		Ok(())
	}
}

#[derive(Clone)]
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
				'}' => {
					if let Some(j) = open {
						if let Ok(token) = (&format[j..=i]).parse() {
							if k != j {
								tokens.push(Token::Literal(format[k..j].into()))
							};
							tokens.push(token);
							k = i + 1;
						}
					}
				}
				_ => {}
			}
		}
		if k != format.len() {
			tokens.push(Token::Literal(format[k..].into()))
		};

		tokens
	}
}

#[derive(Clone)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub enum TimeFormatToken {
	Numeric(Numeric, Pad, bool),
	Literal(String),
	Plural,
}

impl TimeFormatToken {
	pub fn parse(mut format: &str) -> Vec<TimeFormatToken> {
		let res: PResult<Vec<TimeFormatToken>> = repeat(
			0..,
			alt((
				preceded(
					"%",
					(opt(one_of('*')), opt(one_of(['-', '_', '0'])), opt(any)).map(Self::identify),
				),
				take_until(0.., "%").map(|s: &str| TimeFormatToken::Literal(s.into())),
				(peek(any), rest).map(|(_, s): (char, &str)| TimeFormatToken::Literal(s.into())),
			)),
		)
		.parse_next(&mut format);
		res.unwrap()
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
				if let Some(s) = star {
					l.push(s)
				};
				if let Some(f) = flag {
					l.push(f)
				};
				if let Some(c) = spec {
					l.push(c)
				};
				TimeFormatToken::Literal(l)
			}
		}
	}
}

#[derive(Clone)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub enum Numeric {
	Year,
	Month,
	Day,
	Hour,
	Minute,
	Second,
}

#[derive(Clone)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub enum Pad {
	Zero,
	Space,
	None,
}

#[derive(Clone)]
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

#[derive(Copy, Clone, Default)]
pub struct SessionId {
	index: usize,
	len: usize,
	infinite: bool,
	pub iter_no: u64,
	pub total_iter: u64,
}

impl SessionId {
	pub fn new(sessions: &[Session], iterations: Option<u64>) -> Self {
		SessionId {
			index: 0,
			len: sessions.len(),
			infinite: iterations.is_none(),
			iter_no: 0,
			total_iter: iterations.unwrap_or(0),
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

	pub fn jump(&self, idx: usize) -> SessionId {
		SessionId {
			index: idx,
			..*self
		}
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
		assert_eq!(
			&Token::parse("{cyan}{time}{end}\n"),
			&[
				Token::Color(Color::Cyan),
				Token::Time,
				Token::Color(Color::End),
				Token::Literal("\n".into()),
			]
		);
		assert_eq!(
			&Token::parse("String with {time} with some text ahead."),
			&[
				Token::Literal("String with ".into()),
				Token::Time,
				Token::Literal(" with some text ahead.".into())
			]
		);
		assert_eq!(
			&Token::parse("}}{}{{}{}}}{{}{{}}}"),
			&[Token::Literal("}}{}{{}{}}}{{}{{}}}".into())]
		);
		assert_eq!(
			&Token::parse("{time} text {time}"),
			&[Token::Time, Token::Literal(" text ".into()), Token::Time,]
		);
	}

	#[test]
	fn parse_time_format() {
		assert_eq!(
			&TimeFormatToken::parse("%H:%M:%S"),
			&[
				TimeFormatToken::Numeric(Numeric::Hour, Pad::Zero, false),
				TimeFormatToken::Literal(":".into()),
				TimeFormatToken::Numeric(Numeric::Minute, Pad::Zero, false),
				TimeFormatToken::Literal(":".into()),
				TimeFormatToken::Numeric(Numeric::Second, Pad::Zero, false),
			]
		);
		assert_eq!(
			&TimeFormatToken::parse("%L:%M:%S"),
			&[
				TimeFormatToken::Literal("%L".into()),
				TimeFormatToken::Literal(":".into()),
				TimeFormatToken::Numeric(Numeric::Minute, Pad::Zero, false),
				TimeFormatToken::Literal(":".into()),
				TimeFormatToken::Numeric(Numeric::Second, Pad::Zero, false),
			]
		);
		assert_eq!(
			&TimeFormatToken::parse("%H:%M:%"),
			&[
				TimeFormatToken::Numeric(Numeric::Hour, Pad::Zero, false),
				TimeFormatToken::Literal(":".into()),
				TimeFormatToken::Numeric(Numeric::Minute, Pad::Zero, false),
				TimeFormatToken::Literal(":".into()),
				TimeFormatToken::Literal("%".into()),
			]
		);
		assert_eq!(
			&TimeFormatToken::parse("%_H:%-M:%S"),
			&[
				TimeFormatToken::Numeric(Numeric::Hour, Pad::Space, false),
				TimeFormatToken::Literal(":".into()),
				TimeFormatToken::Numeric(Numeric::Minute, Pad::None, false),
				TimeFormatToken::Literal(":".into()),
				TimeFormatToken::Numeric(Numeric::Second, Pad::Zero, false),
			]
		);
		assert_eq!(
			&TimeFormatToken::parse("%H:%*-M:%S"),
			&[
				TimeFormatToken::Numeric(Numeric::Hour, Pad::Zero, false),
				TimeFormatToken::Literal(":".into()),
				TimeFormatToken::Numeric(Numeric::Minute, Pad::None, true),
				TimeFormatToken::Literal(":".into()),
				TimeFormatToken::Numeric(Numeric::Second, Pad::Zero, false),
			]
		);
		assert_eq!(
			&TimeFormatToken::parse("%*-Hh %*-Mm %-Ss"),
			&[
				TimeFormatToken::Numeric(Numeric::Hour, Pad::None, true),
				TimeFormatToken::Literal("h ".into()),
				TimeFormatToken::Numeric(Numeric::Minute, Pad::None, true),
				TimeFormatToken::Literal("m ".into()),
				TimeFormatToken::Numeric(Numeric::Second, Pad::None, false),
				TimeFormatToken::Literal("s".into()),
			]
		);
	}
}
