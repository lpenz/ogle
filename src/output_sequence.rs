// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use color_eyre::Result;
use console::Term;
use std::io::Write;
use std::process::ExitStatus;
use tracing::instrument;

use crate::cli::Cli;
use crate::misc::term_clear_line;
use crate::misc::term_width;
use crate::output_trait::Output;
use crate::progbar;
use crate::timewrap::Duration;
use crate::timewrap::Instant;

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
    term: Term,
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
            term: Term::stdout(),
            width: 80,
            state: Default::default(),
            start: Default::default(),
            sleep_duration: Default::default(),
            run_duration: None,
            refresh: Duration::milliseconds(250),
            lines: vec![],
            iline: 0,
            already_different: true,
            ispinner: 0,
            commandline: Default::default(),
        }
    }
}

impl OutputSequence {
    #[instrument(level = "debug")]
    pub fn new(cli: &Cli) -> Self {
        let term = Term::stdout();
        let width = term_width(&term);
        let commandline = cli.command.join(" ");
        Self {
            term,
            width,
            start: Instant::now(),
            sleep_duration: Duration::seconds(cli.period as i64),
            commandline,
            ..Default::default()
        }
    }

    fn write_line(&mut self, line: &str) -> Result<()> {
        self.term.write_all(line.as_bytes())?;
        self.term.write_all(b"\n")?;
        Ok(())
    }

    fn write_line_scroll(&mut self, line: &str) -> Result<()> {
        self.write_line(line)?;
        self.term.write_all(b"\n")?;
        self.term.flush()?;
        Ok(())
    }

    fn write_all_lines(&mut self) -> Result<()> {
        let mut lines = std::mem::take(&mut self.lines);
        for line in &lines {
            self.write_line(line)?;
        }
        self.term.flush()?;
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
    fn run_start(&mut self) -> Result<()> {
        let now = Instant::now();
        if self.run_duration.is_none() {
            // First execution
            self.write_line(&ofmt!(&now, "first execution"))?;
            self.write_line_scroll(&format!("+ {}", self.commandline))?;
        }
        self.state = State::Running;
        self.start = now;
        // Let's refresh the terminal width every time we start
        // running the program.
        self.width = term_width(&self.term);
        self.iline = 0;
        self.tick()?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    fn run_end(&mut self, exitstatus: &ExitStatus) -> Result<()> {
        let now = Instant::now();
        self.run_duration = Some(&now - &self.start);
        self.state = State::Sleeping;
        self.start = now;
        self.iline = 0;
        self.already_different = false;
        self.tick()?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    fn out_line(&mut self, line: String) -> Result<()> {
        self.iline += 1;
        if self.iline <= self.lines.len() && self.lines[self.iline - 1] == line {
            // Same as last execution, keep going
            return self.tick();
        }
        // Something is different
        term_clear_line(&self.term)?;
        if !self.already_different {
            self.write_line(&ofmt!(&Instant::now(), "changed"))?;
            self.write_line(&format!("+ {}", self.commandline))?;
            self.lines.truncate(self.iline - 1);
            self.write_all_lines()?;
            self.already_different = true;
        }
        self.write_line_scroll(&line)?;
        self.lines.push(line);
        self.tick()?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    fn err_line(&mut self, line: String) -> Result<()> {
        self.out_line(line)
    }

    // #[instrument(level = "debug", skip(self))]
    fn tick(&mut self) -> Result<()> {
        let now = Instant::now();
        match self.state {
            State::Starting => {}
            State::Sleeping => {
                term_clear_line(&self.term)?;
                self.term.write_line(&progbar::progbar_sleeping(
                    &self.start,
                    &now,
                    &self.start,
                    &self.sleep_duration,
                )?)?;
            }
            State::Running => {
                term_clear_line(&self.term)?;
                let spinner = self.spinner_get();
                self.term.write_line(&progbar::progbar_running(
                    self.width,
                    &now,
                    &now,
                    &self.start,
                    &self.run_duration,
                    &self.refresh,
                    spinner,
                )?)?;
            }
        }
        Ok(())
    }
}
