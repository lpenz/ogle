// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant(chrono::DateTime<chrono::Local>);

impl Instant {
    pub fn now() -> Self {
        Self(chrono::offset::Local::now())
    }

    pub fn elapsed(&self) -> Duration {
        &Instant::now() - self
    }
}

impl fmt::Display for Instant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.format("%Y-%m-%d %H:%M:%S"))
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

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

impl From<Duration> for std::time::Duration {
    fn from(duration: Duration) -> Self {
        duration.0.to_std().unwrap()
    }
}
