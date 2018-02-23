use std;
use std::fmt;
use std::path;

use time;


/// The time format used for item timestamps.
const TIME_FORMAT: &str = "%Y-%m-%d %H:%M";


/// Extracts the modified timestamp from a file.
///
/// If no meta data can be extracted, the current time is returned.
///
/// # Arguments
/// *  `path` - The path of the file.
pub fn timestamp<P: AsRef<path::Path>>(path: &P) -> std::time::SystemTime {
    let path: &path::Path = path.as_ref();
    path.metadata().and_then(|meta| meta.modified()).unwrap_or(
        std::time::SystemTime::now(),
    )
}


/// Converts a system time to a time spec.
///
/// # Arguments
/// *  `st` - The system time to convert.
pub fn system_time_to_timespec(st: std::time::SystemTime) -> time::Timespec {
    // If the source is before the Unix epoch, we must manually invert it
    let (sec, ns) =
        if let Ok(duration) = st.duration_since(std::time::UNIX_EPOCH) {
            (duration.as_secs() as i64, duration.subsec_nanos() as i32)
        } else {
            let duration = std::time::UNIX_EPOCH.duration_since(st).unwrap();
            (
                -(duration.as_secs() as i64),
                -(duration.subsec_nanos() as i32),
            )
        };

    time::Timespec::new(sec, ns).into()
}


/// A wrapped timestamp.
#[derive(Clone, Debug)]
pub struct Timestamp(time::Tm);

impl Timestamp {
    /// Returns the year.
    pub fn year(&self) -> i32 {
        return self.0.tm_year + 1900;
    }

    /// Returns the month.
    pub fn month(&self) -> i32 {
        return self.0.tm_mon + 1;
    }

    /// Returns the day.
    pub fn day(&self) -> i32 {
        return self.0.tm_mday;
    }
}


impl fmt::Display for Timestamp {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.0.strftime(TIME_FORMAT).unwrap())
    }
}


impl From<time::Tm> for Timestamp {
    /// Converts a calendar time by wrapping it.
    ///
    /// # Arguments
    /// *  `source` - The source time.
    fn from(source: time::Tm) -> Self {
        Timestamp(source)
    }
}

impl From<(i32, i32, i32)> for Timestamp {
    /// Converts a year, month and day.
    ///
    /// Other fields will be set to 0.
    ///
    /// # Arguments
    /// *  `year` - The year.
    /// *  `month` - The month.
    /// *  `day` - The day.
    fn from((year, month, day): (i32, i32, i32)) -> Self {
        (year, month, day, 0, 0, 0).into()
    }
}

impl From<(i32, i32, i32, i32, i32, i32)> for Timestamp {
    /// Converts a year, month, day, hour, minute and second.
    ///
    /// Other fields will be set to zero.
    ///
    /// # Arguments
    /// *  `year` - The year.
    /// *  `month` - The month.
    /// *  `day` - The day.
    /// *  `hour` - The hour.
    /// *  `min` - The minute.
    /// *  `sec` - The second.
    fn from(
        (year, month, day, hour, min, sec): (i32, i32, i32, i32, i32, i32),
    ) -> Self {
        let tm_year = year - 1900;
        let tm_mon = month - 1;
        let tm_mday = day;
        let tm_hour = hour;
        let tm_min = min;
        let tm_sec = sec;
        Timestamp(time::Tm {
            tm_year,
            tm_mon,
            tm_mday,
            tm_hour,
            tm_min,
            tm_sec,
            tm_nsec: 0,
            tm_yday: 0,
            tm_wday: 0,
            tm_isdst: -1,
            tm_utcoff: 0,
        })
    }
}

impl From<std::time::SystemTime> for Timestamp {
    /// Converts a system time.
    ///
    /// # Arguments
    /// *  `source` - The system time to convert.
    fn from(source: std::time::SystemTime) -> Self {
        system_time_to_timespec(source).into()
    }
}

impl From<time::Timespec> for Timestamp {
    /// Converts a timestamp.
    ///
    /// # Arguments
    /// *  `source` - The timestamp.
    fn from(source: time::Timespec) -> Self {
        Timestamp(time::at(source))
    }
}


impl AsRef<time::Tm> for Timestamp {
    fn as_ref(&self) -> &time::Tm {
        &self.0
    }
}
