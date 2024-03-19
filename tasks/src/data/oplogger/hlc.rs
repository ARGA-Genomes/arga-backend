

// MMDHmSssnn
// months = 16bit = 65536 (5461 years)
// days = 6bit = 64
// hours = 6bit = 64
// minutes = 6bit = 64
// seconds = 6bit = 64
// milliseconds = 10bit = 1024
// logical = 14bit = 16384
//
// hlc paper has:
// timestamp = 48bits (microsecond granularity)
//     32bits = seconds from epoc (136 years)
//     16bits = fractional second
// logical = 16 bits


#[derive(Clone, Debug, Eq, Hash)]
pub struct HybridTimestamp(u64);

impl HybridTimestamp {
    pub fn new(timestamp: u64) -> Self {
        Self(timestamp)
    }

    pub fn with_logical(timestamp: u64, logical: u16) -> Self {
        Self(((timestamp >> 14) << 14) | logical as u64)
    }

    pub fn inc(&self) -> HybridTimestamp {
        Self::with_logical(self.0, self.logical() + 1)
    }

    pub fn months(&self) -> u16 {
        ((self.0 >> 48) & 0b11111111_11111111) as u16
    }

    pub fn days(&self) -> u8 {
        ((self.0 >> 42) & 0b00111111) as u8
    }

    pub fn hours(&self) -> u8 {
        ((self.0 >> 36) & 0b00111111) as u8
    }

    pub fn minutes(&self) -> u8 {
        ((self.0 >> 30) & 0b00111111) as u8
    }

    pub fn seconds(&self) -> u8 {
        ((self.0 >> 24) & 0b00111111) as u8
    }

    pub fn milliseconds(&self) -> u16 {
        ((self.0 >> 14) & 0b00000011_11111111) as u16
    }

    pub fn logical(&self) -> u16 {
        (self.0 & 0b00111111_11111111) as u16
    }


    pub fn as_u64(&self) -> u64 {
        self.0
    }
}


use chrono::{DateTime, Utc, TimeZone, Datelike, Timelike};


impl From<HybridTimestamp> for DateTime<Utc> {
    fn from(ts: HybridTimestamp) -> Self {
        let years = 2020 + (ts.months() / 12);
        let months = ts.months() % 12;

        Utc.ymd(years as i32, months as u32, ts.days() as u32)
           .and_hms_milli(ts.hours() as u32, ts.minutes() as u32, ts.seconds() as u32, ts.milliseconds() as u32)
    }
}


impl From<DateTime<Utc>> for HybridTimestamp {
    fn from(source: DateTime<Utc>) -> Self {
        let months: u64 = ((source.year() - 2020) as u64) + source.month0() as u64;
        let timestamp = (months << 48)
            | ((source.day() as u64) << 42)
            | ((source.hour() as u64) << 36)
            | ((source.minute() as u64) << 30)
            | ((source.second() as u64) << 24)
            | (((source.nanosecond() / 1000_000) as u64) << 14);

        Self(timestamp)
    }
}


impl std::fmt::Display for HybridTimestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}


impl Ord for HybridTimestamp {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for HybridTimestamp {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for HybridTimestamp {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl From<HybridTimestamp> for u64 {
    fn from(source: HybridTimestamp) -> u64 {
        source.0
    }
}
