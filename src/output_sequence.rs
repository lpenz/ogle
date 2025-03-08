// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use color_eyre::Result;
use std::process::ExitStatus;
use tracing::instrument;

use crate::cli::Cli;
use crate::output_trait::Output;
use crate::progbar;
use crate::sys_api::Sys;
use crate::sys_api::SysApi;
use crate::time_wrapper::Duration;
use crate::time_wrapper::Instant;

const SPINNERS: [char; 4] = ['/', '-', '\\', '|'];

#[derive(Debug, Default)]
pub enum State {
    #[default]
    Starting,
    Sleeping,
    Running,
}

#[derive(Debug)]
pub struct OutputSequence {
    width: usize,
    state: State,
    start: Instant,
    sleep_duration: Duration,
    run_duration: Option<Duration>,
    refresh: Duration,
    lines: Vec<String>,
    iline: usize,
    already_different: bool,
    ispinner: usize,
    commandline: String,
}

impl Default for OutputSequence {
    fn default() -> Self {
        Self {
            width: 80,
            state: State::default(),
            start: Instant::default(),
            sleep_duration: Duration::default(),
            run_duration: None,
            refresh: Duration::milliseconds(250),
            lines: vec![],
            iline: 0,
            already_different: true,
            ispinner: 0,
            commandline: String::default(),
        }
    }
}

impl OutputSequence {
    #[instrument(level = "debug")]
    pub fn new(sys: &Sys, cli: &Cli) -> Self {
        let width = sys.width();
        let commandline = cli.command.join(" ");
        Self {
            width,
            start: sys.now(),
            sleep_duration: Duration::seconds(i64::from(cli.period)),
            commandline,
            ..Default::default()
        }
    }

    fn log_all_lines(&mut self, sys: &mut Sys) -> Result<()> {
        let mut lines = std::mem::take(&mut self.lines);
        for line in &lines {
            sys.log_line(line)?;
        }
        self.lines = std::mem::take(&mut lines);
        Ok(())
    }

    fn spinner_get(&mut self) -> char {
        let spinner = SPINNERS[self.ispinner];
        self.ispinner = (self.ispinner + 1) % SPINNERS.len();
        spinner
    }
}

impl Output for OutputSequence {
    #[instrument(level = "debug", fields(self=?self.state))]
    fn run_start(&mut self, sys: &mut Sys) -> Result<()> {
        let now = sys.now();
        if self.run_duration.is_none() {
            // First execution
            sys.log_line(&ofmt!(&now, "first execution"))?;
            sys.log_line(&format!("+ {}", self.commandline))?;
        }
        self.state = State::Running;
        self.start = now;
        // Let's refresh the terminal width every time we start
        // running the program.
        self.width = sys.width();
        self.iline = 0;
        self.tick(sys)?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    fn run_end(&mut self, sys: &mut Sys, exitstatus: ExitStatus) -> Result<()> {
        let now = sys.now();
        self.run_duration = Some(&now - &self.start);
        self.state = State::Sleeping;
        self.start = now;
        self.iline = 0;
        self.already_different = false;
        self.tick(sys)?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    fn out_line(&mut self, sys: &mut Sys, line: String) -> Result<()> {
        self.iline += 1;
        if self.iline <= self.lines.len() && self.lines[self.iline - 1] == line {
            // Same as last execution, keep going
            return self.tick(sys);
        }
        // Something is different
        if !self.already_different {
            sys.log_line(&ofmt!(&sys.now(), "changed"))?;
            sys.log_line(&format!("+ {}", self.commandline))?;
            self.lines.truncate(self.iline - 1);
            self.log_all_lines(sys)?;
            self.already_different = true;
        }
        sys.log_line(&line)?;
        self.lines.push(line);
        self.tick(sys)?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    fn err_line(&mut self, sys: &mut Sys, line: String) -> Result<()> {
        self.out_line(sys, line)
    }

    // #[instrument(level = "debug", skip(self))]
    fn tick(&mut self, sys: &mut Sys) -> Result<()> {
        let now = sys.now();
        match self.state {
            State::Starting => {}
            State::Sleeping => {
                sys.update_status(&progbar::progbar_sleeping(
                    &self.start,
                    &now,
                    &self.start,
                    &self.sleep_duration,
                ))?;
            }
            State::Running => {
                let spinner = self.spinner_get();
                sys.update_status(&progbar::progbar_running(
                    self.width,
                    &now,
                    &now,
                    &self.start,
                    self.run_duration.as_ref(),
                    &self.refresh,
                    spinner,
                )?)?;
            }
        }
        Ok(())
    }
}
