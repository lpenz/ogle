// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use crate::sys_api::SysApi;
use crate::time_wrapper::Instant;

/// [`SysApi`] implementation of the real environment
#[derive(Default, Debug, Clone)]
pub struct Sys {}

impl Sys {
    pub fn now(&self) -> Instant {
        Instant::from(chrono::offset::Utc::now())
    }
}

impl SysApi for Sys {
    fn now(&self) -> Instant {
        self.now()
    }
}
