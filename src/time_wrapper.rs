// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant(chrono::DateTime<chrono::Utc>);

impl Instant {
    pub fn now() -> Self {
        Self(chrono::offset::Utc::now())
    }

    pub fn epoch() -> Self {
        Instant(chrono::DateTime::UNIX_EPOCH)
    }

    pub fn elapsed(&self) -> Duration {
        &Instant::now() - self
    }
}

impl Default for Instant {
    fn default() -> Self {
        Self::epoch()
    }
}

impl fmt::Display for Instant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use UTC in tests, locatime in prod
        #[cfg(not(test))]
        let dt = chrono::DateTime::<chrono::Local>::from(self.0);
        #[cfg(test)]
        let dt = self.0;
        write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S"))
    }
}

impl std::ops::Add<&Duration> for &Instant {
    type Output = Instant;
    fn add(self, other: &Duration) -> Instant {
        Instant(self.0 + other.0)
    }
}

impl std::ops::Sub for &Instant {
    type Output = Duration;
    fn sub(self, other: Self) -> Self::Output {
        Duration(self.0 - other.0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration(chrono::Duration);

impl Duration {
    pub const fn seconds(value: i64) -> Self {
        Self(chrono::Duration::seconds(value))
    }

    pub const fn milliseconds(value: i64) -> Self {
        Self(chrono::Duration::milliseconds(value))
    }

    pub const fn num_seconds(&self) -> i64 {
        self.0.num_seconds()
    }

    pub const fn num_milliseconds(&self) -> i64 {
        self.0.num_milliseconds()
    }
}

impl Default for Duration {
    fn default() -> Self {
        Self::milliseconds(1)
    }
}

impl From<Duration> for std::time::Duration {
    fn from(duration: Duration) -> Self {
        duration.0.to_std().unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_instant() {
        let ten = Duration::seconds(10);
        let now = Instant::now();
        assert!(now.elapsed() < ten);
        let now2 = Instant::now();
        assert!(&now2 - &now < ten);
        let now3 = &now2 + &ten;
        assert!(&now3 > &now2);
    }

    #[test]
    fn print_instant() {
        let epoch = Instant::epoch();
        let string = format!("{}", epoch);
        // Let's just test the start, as the time can vary with the local timezone.
        assert_eq!(string, "1970-01-01 00:00:00");
    }

    #[test]
    fn basic_duration() {
        assert_eq!(Duration::seconds(10).num_seconds(), 10);
        assert_eq!(Duration::milliseconds(10).num_milliseconds(), 10);
    }
}
