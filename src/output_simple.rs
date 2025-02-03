// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use color_eyre::Result;
use console::Term;
use std::process::ExitStatus;
use tracing::instrument;

use crate::output_trait::Output;
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
    pub fn new() -> OutputSimple {
        OutputSimple::default()
    }
}

impl Output for OutputSimple {
    #[instrument(level = "debug", fields(self=?self.start))]
    fn run_start(&mut self) -> Result<()> {
        let now = Instant::now();
        self.start = Some(now);
        let msg = format!("[ogle] {} execution start", Instant::now());
        self.term.write_line(&msg)?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    fn run_end(&mut self, exitstatus: &ExitStatus) -> Result<()> {
        let msg = format!(
            "[ogle] {} execution ended with {:?}",
            Instant::now(),
            exitstatus
        );
        self.term.write_line(&msg)?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    fn out_line(&mut self, line: String) -> Result<()> {
        self.term.write_line(&line)?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    fn err_line(&mut self, line: String) -> Result<()> {
        self.term.write_line(&line)?;
        Ok(())
    }

    // #[instrument(level = "debug", skip(self))]
    fn tick(&mut self) -> Result<()> {
        Ok(())
    }
}
