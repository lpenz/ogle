// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use color_eyre::Result;
use console::Term;
use std::process::ExitStatus;
use tracing::instrument;

use crate::cli::Cli;
use crate::misc::term_clear_line;
use crate::misc::term_width;
use crate::output_trait::Output;
use crate::progbar;
use crate::timewrap::Duration;
use crate::timewrap::Instant;

#[derive(Debug, Default)]
pub enum State {
    #[default]
    Starting,
    Sleeping {
        start: Instant,
    },
    Running {
        start: Instant,
        width: usize,
    },
}

#[derive(Debug)]
pub struct OutputSequence {
    state: State,
    term: Term,
    sleep_duration: Duration,
    refresh: Duration,
    last_run: Option<Duration>,
}

impl OutputSequence {
    #[instrument(level = "debug")]
    pub fn new(cli: &Cli) -> Self {
        Self {
            state: State::default(),
            term: Term::stdout(),
            sleep_duration: Duration::seconds(cli.period as i64),
            refresh: Duration::milliseconds(250),
            last_run: None,
        }
    }
}

impl Output for OutputSequence {
    #[instrument(level = "debug", fields(self=?self.state))]
    fn run_start(&mut self) -> Result<()> {
        let now = Instant::now();
        self.state = State::Running {
            start: now,
            width: term_width(&self.term),
        };
        self.tick()?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    fn run_end(&mut self, exitstatus: &ExitStatus) -> Result<()> {
        let now = Instant::now();
        self.state = State::Sleeping { start: now };
        self.tick()?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    fn out_line(&mut self, line: String) -> Result<()> {
        self.term.write_line(&line)?;
        self.tick()?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    fn err_line(&mut self, line: String) -> Result<()> {
        self.term.write_line(&line)?;
        self.tick()?;
        Ok(())
    }

    // #[instrument(level = "debug", skip(self))]
    fn tick(&mut self) -> Result<()> {
        let now = Instant::now();
        match self.state {
            State::Starting => {}
            State::Sleeping { start } => {
                term_clear_line(&self.term)?;
                self.term.write_line(&progbar::progbar_sleeping(
                    &start,
                    &now,
                    &start,
                    &self.sleep_duration,
                )?)?;
            }
            State::Running { start, width } => {
                term_clear_line(&self.term)?;
                // let elapsed = &now - &start;
                self.term.write_line(&progbar::progbar_running(
                    width,
                    &now,
                    &now,
                    &start,
                    &self.last_run,
                    &self.refresh,
                    ' ',
                )?)?;
            }
        }
        Ok(())
    }
}
