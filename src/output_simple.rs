// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use color_eyre::Result;
use console::Term;
use std::process::ExitStatus;
use tracing::instrument;

use crate::output_trait::Output;
use crate::sys_api::SysApi;
use crate::time_wrapper::Instant;

#[derive(Debug)]
pub struct OutputSimple {
    start: Option<Instant>,
    term: Term,
}

impl Default for OutputSimple {
    fn default() -> OutputSimple {
        OutputSimple {
            start: None,
            term: Term::stdout(),
        }
    }
}

impl OutputSimple {
    #[instrument(level = "debug")]
    pub fn new() -> Self {
        OutputSimple::default()
    }
}

impl Output for OutputSimple {
    #[instrument(level = "debug", fields(self=?self.start))]
    fn run_start<Sys: SysApi + 'static>(&mut self, sys: &Sys) -> Result<()> {
        let now = sys.now();
        self.start = Some(now);
        let msg = format!("[ogle] {} execution start", sys.now());
        self.term.write_line(&msg)?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    fn run_end<Sys: SysApi>(&mut self, sys: &Sys, exitstatus: ExitStatus) -> Result<()> {
        let msg = format!("[ogle] {} execution ended with {:?}", sys.now(), exitstatus);
        self.term.write_line(&msg)?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    fn out_line<Sys: SysApi + 'static>(&mut self, sys: &Sys, line: String) -> Result<()> {
        self.term.write_line(&line)?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    fn err_line<Sys: SysApi + 'static>(&mut self, sys: &Sys, line: String) -> Result<()> {
        self.term.write_line(&line)?;
        Ok(())
    }

    // #[instrument(level = "debug", skip(self))]
    fn tick<Sys: SysApi + 'static>(&mut self, _sys: &Sys) -> Result<()> {
        Ok(())
    }
}
