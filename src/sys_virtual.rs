// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use color_eyre::Result;
use tokio::process::Command;

use crate::stream::Streamer;
use crate::sys::SysApi;
use crate::time_wrapper::Duration;
use crate::time_wrapper::Instant;

/// [`SysApi`] implementation of a virtual environment
#[derive(Debug, Default)]
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

    fn run_command(&self, command: Command, refresh_delay: Duration) -> Result<Streamer> {
        crate::stream::Streamer::new(command, refresh_delay)
    }
}
