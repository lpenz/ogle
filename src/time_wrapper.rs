// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Wrapper for time [`Instant`] and [`Duration`] abstractions.
//!
//! We have a couple of options when choosing a time create in rust -
//! so creating a wrapper module makes sense, as it allows us to
//! change the underlying crate while minimizing changes to users.
//!
//! We use [`chrono`] as the wrapped crate at the moment.
use std::fmt;

// Instant ///////////////////////////////////////////////////////////

type InstantInner = chrono::DateTime<chrono::Utc>;

/// A specific instant in time.
///
/// Wraps [`chrono::DateTime<chrono::Utc>`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant(InstantInner);

impl Instant {
    #[cfg(test)]
    pub fn incr(&mut self) -> Self {
        let me = *self;
        *self = &me + &Duration::seconds(1);
        me
    }
}

impl Default for Instant {
    fn default() -> Self {
        Instant(chrono::DateTime::UNIX_EPOCH)
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

impl From<InstantInner> for Instant {
    fn from(dt: InstantInner) -> Self {
        Self(dt)
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

// Duration //////////////////////////////////////////////////////////

/// A time duration - the difference between two [`Instant`]s.
///
/// Wraps [`chrono::Duration`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration(chrono::Duration);

impl Duration {
    /// An absurd duration (a millenia) that is safe to add/subtract
    /// without overflowing the inner type.
    pub const INFINITE: Self = Self::seconds(3600 * 24 * 365 * 1000);

    /// Makes a new Duration with the given number of seconds.
    ///
    /// Wraps [`chrono::Duration::seconds`].
    pub const fn seconds(value: i64) -> Self {
        Self(chrono::Duration::seconds(value))
    }

    /// Makes a new Duration with the given number of milliseconds.
    ///
    /// Wraps [`chrono::Duration::milliseconds`].
    pub const fn milliseconds(value: i64) -> Self {
        Self(chrono::Duration::milliseconds(value))
    }

    /// Returns the total number of whole seconds in the `Duration`.
    ///
    /// Wraps [`chrono::Duration::num_seconds`].
    pub const fn num_seconds(&self) -> i64 {
        self.0.num_seconds()
    }

    /// Returns the total number of whole seconds in the `Duration`.
    ///
    /// Wraps [`chrono::Duration::num_milliseconds`].
    pub const fn num_milliseconds(&self) -> i64 {
        self.0.num_milliseconds()
    }
}

impl Default for Duration {
    fn default() -> Self {
        Self::milliseconds(0)
    }
}

impl From<Duration> for std::time::Duration {
    fn from(duration: Duration) -> Self {
        duration.0.to_std().unwrap_or_default()
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
        let ten = Duration::seconds(10);
        let sys = SysReal::default();
        let now = sys.now();
        let now2 = sys.now();
        assert!(&now2 - &now < ten);
        let now3 = &now2 + &ten;
        assert!(&now3 > &now2);
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
        assert_eq!(Duration::seconds(10).num_seconds(), 10);
        assert_eq!(Duration::milliseconds(10).num_milliseconds(), 10);
        assert_eq!(
            &Instant::default() + &Duration::default(),
            Instant::default()
        );
    }
}
