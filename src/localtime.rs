// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use std::fmt;

#[derive(Debug, Clone)]
pub struct LocalTime(chrono::DateTime<chrono::Local>);

impl LocalTime {
    pub fn now() -> Self {
        Self(chrono::offset::Local::now())
    }
}

impl fmt::Display for LocalTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.format("%Y-%m-%d %H:%M:%S"))
    }
}
