// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use color_eyre::Result;
use console::Term;
use std::io::Write;
use tokio::process::Command;

use crate::stream::Streamer;
use crate::sys::SysApi;
use crate::time_wrapper::Duration;
use crate::time_wrapper::Instant;

/// [`SysApi`] implementation of the real environment
#[derive(Debug)]
pub struct SysReal {
    term: Term,
    status_visible: bool,
}

impl Default for SysReal {
    fn default() -> Self {
        Self {
            term: Term::stdout(),
            status_visible: false,
        }
    }
}

impl SysReal {
    fn clear_line(&mut self) -> Result<()> {
        self.term.move_cursor_up(1)?;
        self.term.clear_line()?;
        Ok(())
    }

    fn write_line(&mut self, line: &str) -> Result<()> {
        if self.status_visible {
            self.clear_line()?;
        }
        self.term.write_all(line.as_bytes())?;
        self.term.write_all(b"\n")?;
        self.term.flush()?;
        Ok(())
    }
}

impl SysApi for SysReal {
    fn now(&self) -> Instant {
        Instant::from(chrono::offset::Utc::now())
    }

    fn width(&self) -> usize {
        if let Some((_, w)) = self.term.size_checked() {
            w as usize
        } else {
            80
        }
    }

    fn log_line(&mut self, line: &str) -> Result<()> {
        self.write_line(line)?;
        self.status_visible = false;
        Ok(())
    }

    fn update_status(&mut self, status: &str) -> Result<()> {
        self.write_line(status)?;
        self.status_visible = true;
        Ok(())
    }

    fn run_command(&self, command: Command, refresh_delay: Duration) -> Result<Streamer> {
        crate::stream::Streamer::new(command, refresh_delay)
    }
}
