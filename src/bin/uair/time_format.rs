use std::time::Duration;
use humantime::format_duration;

pub enum TimeFormat {
	Humantime,
	MinSec,
}

impl TimeFormat {
	pub fn format_duration(&self, duration: Duration) -> String {
		match &self {
			TimeFormat::Humantime => Self::humantime(duration),
			TimeFormat::MinSec => Self::min_sec(duration),
		}
	}

	fn humantime(duration: Duration) -> String {
		format!("{}", format_duration(duration))
	}

	/// Formats duration as `min:sec` or `hour:min:sec` in case more then 1 hour.
	///
	/// # Examples
	/// ```
	/// let formatted = time_format.min_sec(Duration::from_secs(1234);
	/// 
	/// assert_eq!("20:34", formatted);
	/// ```
	fn min_sec(duration: Duration) -> String {
		let secs = duration.as_secs();

		if secs >= 86400 {
			panic!("Duration should not exceed 24 hours: {}", format_duration(duration));
		}

		let hours = secs / 3600;
		let minutes = (secs / 60) % 60;
		let seconds = secs % 60;

		if secs >= 3600 {
			return format!("{hours:0>2}:{minutes:0>2}:{seconds:0>2}");
		}

		format!("{minutes:0>2}:{seconds:0>2}")
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn valid_min_sec() {
		assert_eq!("00:00", TimeFormat::min_sec(Duration::from_secs(0)));
		assert_eq!("00:11", TimeFormat::min_sec(Duration::from_secs(11)));
		assert_eq!("03:23", TimeFormat::min_sec(Duration::from_secs(203)));
		assert_eq!("59:59", TimeFormat::min_sec(Duration::from_secs(3599)));
		assert_eq!("01:00:11", TimeFormat::min_sec(Duration::from_secs(3611)));
		assert_eq!("11:00:00", TimeFormat::min_sec(Duration::from_secs(39600)));
		assert_eq!("23:59:59", TimeFormat::min_sec(Duration::from_secs(86399)));
	}

	#[test]
	#[should_panic]
	fn min_sec_overflowed() {
		TimeFormat::min_sec(Duration::from_secs(86400)); // one day
	}
}
