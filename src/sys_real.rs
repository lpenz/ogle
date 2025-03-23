// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use color_eyre::Result;

use crate::sys::SysApi;
use crate::term_wrapper::*;
use crate::time_wrapper::Instant;

/// [`SysApi`] implementation of the real environment
#[derive(Debug, Default)]
pub struct SysReal {
    status_visible: bool,
}

impl SysReal {
    fn clear_line(&mut self) -> Result<()> {
        move_cursor_up(1)?;
        clear_line()?;
        Ok(())
    }

    fn write_line(&mut self, line: &str) -> Result<()> {
        if self.status_visible {
            self.clear_line()?;
        }
        write_all(line.as_bytes())?;
        write_all(b"\n")?;
        flush()?;
        Ok(())
    }
}

impl SysApi for SysReal {
    fn now(&self) -> Instant {
        Instant::from(chrono::offset::Utc::now())
    }

    fn width(&self) -> usize {
        if let Some((_, w)) = size_checked() {
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
}
