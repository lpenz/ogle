// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Wrapper for time [`Instant`] and [`Duration`] abstractions.
//!
//! We have a couple of options when choosing a time create in rust -
//! so creating a wrapper module makes sense, as it allows us to
//! change the underlying crate while minimizing changes to users.
//!
//! We use [`jiff`] as the wrapped crate at the moment.
//!
//! [`Instant::EPOCH`] is the Unix epoch, and is also the value returned by
//! [`Instant::default`].
use std::fmt;

// Instant ///////////////////////////////////////////////////////////

type InstantInner = jiff::Timestamp;

/// A specific instant in time.
///
/// Wraps [`jiff::Timestamp`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant(InstantInner);

impl Instant {
    /// The Unix epoch: 1970-01-01 00:00:00 UTC.
    ///
    /// This is the [`Default`] value for [`Instant`], and the starting point
    /// used by virtual clocks in tests.
    pub const EPOCH: Self = Self(jiff::Timestamp::UNIX_EPOCH);

    /// Returns an [`Instant`] that corresponds to the current wall-clock time.
    pub fn now() -> Self {
        Self(jiff::Timestamp::now())
    }

    #[cfg(test)]
    pub fn incr(&mut self) -> Self {
        let me = *self;
        *self = &me + &Duration::from_secs(1);
        me
    }
}

impl Default for Instant {
    fn default() -> Self {
        Self::EPOCH
    }
}

impl fmt::Display for Instant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use UTC in tests, locatime in prod
        #[cfg(not(test))]
        let dt = self.0.to_zoned_datetime(jiff::tz::TimeZone::system());
        #[cfg(test)]
        let dt = self.0.to_zoned_datetime(jiff::tz::TimeZone::UTC);
        write!(f, "{}", dt.strftime("%Y-%m-%d %H:%M:%S"))
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
        Duration(self.0.duration_since(other.0))
    }
}

// Duration //////////////////////////////////////////////////////////

/// A time duration - the difference between two [`Instant`]s.
///
/// Wraps [`jiff::SignedDuration`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration(jiff::SignedDuration);

impl Duration {
    /// An absurd duration (a millenia) that is safe to add/subtract
    /// without overflowing the inner type.
    pub const INFINITE: Self = Self::from_secs(3600 * 24 * 365 * 1000);

    /// Makes a new Duration with the given number of seconds.
    ///
    /// Wraps [`jiff::SignedDuration::from_secs`].
    pub const fn from_secs(value: i64) -> Self {
        Self(jiff::SignedDuration::from_secs(value))
    }

    /// Makes a new Duration with the given number of milliseconds.
    ///
    /// Wraps [`jiff::SignedDuration::from_millis`].
    pub const fn from_millis(value: i64) -> Self {
        Self(jiff::SignedDuration::from_millis(value))
    }

    /// Returns the total number of whole seconds in the `Duration`.
    ///
    /// Wraps [`jiff::SignedDuration::as_secs`].
    pub const fn as_secs(&self) -> i64 {
        self.0.as_secs()
    }

    /// Returns the total number of whole milliseconds in the `Duration`.
    ///
    /// Wraps [`jiff::SignedDuration::as_millis`].
    pub const fn as_millis(&self) -> i64 {
        self.0.as_millis() as i64
    }
}

impl Default for Duration {
    fn default() -> Self {
        Self::from_millis(0)
    }
}

impl From<Duration> for std::time::Duration {
    fn from(duration: Duration) -> Self {
        std::time::Duration::try_from(duration.0).unwrap_or_default()
    }
}

impl From<Duration> for tokio::time::Interval {
    fn from(duration: Duration) -> Self {
        tokio::time::interval(duration.into())
    }
}

// Tests /////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use super::*;

    use crate::sys::SysApi;
    use crate::sys::SysReal;

    #[test]
    fn basic_instant() {
        let ten = Duration::from_secs(10);
        let sys = SysReal::default();
        let now = sys.now();
        let now2 = sys.now();
        assert!(&now2 - &now < ten);
        let now3 = &now2 + &ten;
        assert!(now3 > now2);
    }

    #[test]
    fn print_instant() {
        let epoch = Instant::default();
        let string = format!("{}", epoch);
        // Let's just test the start, as the time can vary with the local timezone.
        assert_eq!(string, "1970-01-01 00:00:00");
    }

    #[test]
    fn basic_duration() {
        assert_eq!(Duration::from_secs(10).as_secs(), 10);
        assert_eq!(Duration::from_millis(10).as_millis(), 10);
        assert_eq!(
            &Instant::default() + &Duration::default(),
            Instant::default()
        );
    }
}
