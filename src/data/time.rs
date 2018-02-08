use time;


/// The time format used for item timestamps.
pub const TIME_FORMAT: &str = "%Y-%m-%d %H:%M";


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


impl AsRef<time::Tm> for Timestamp {
    fn as_ref(&self) -> &time::Tm {
        &self.0
    }
}
