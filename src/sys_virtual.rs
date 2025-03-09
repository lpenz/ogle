// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use color_eyre::Result;

use crate::sys::SysApi;
use crate::time_wrapper::Instant;

/// [`SysApi`] implementation of a virtual environment
#[derive(Debug, Clone, Default)]
pub struct SysVirtual {
    pub log: Vec<String>,
    pub status: Vec<String>,
    pub now: Instant,
}

impl SysApi for SysVirtual {
    fn now(&self) -> Instant {
        self.now
    }

    fn width(&self) -> usize {
        80
    }

    fn log_line(&mut self, line: &str) -> Result<()> {
        self.log.push(line.into());
        Ok(())
    }

    fn update_status(&mut self, status: &str) -> Result<()> {
        self.status.push(status.into());
        Ok(())
    }
}
