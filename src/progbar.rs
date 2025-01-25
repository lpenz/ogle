// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use color_eyre::Report;
use color_eyre::Result;
use console::Term;

use crate::timewrap::Duration;
use crate::timewrap::Instant;

const SPINNERS: [char; 4] = ['/', '-', '\\', '|'];

// Basic functions:

pub fn progbar_sleeping(
    timestamp: &Instant,
    start: &Instant,
    duration: &Duration,
) -> Result<String> {
    let msg = if duration.num_seconds() > 1 {
        let end = start + duration;
        let left = &end - &Instant::now();
        format!("=> {} sleeping for {}s", timestamp, left.num_seconds() + 1)
    } else {
        format!("=> {} sleeping", timestamp)
    };
    Ok(msg)
}

pub fn progbar_running(
    width: usize,
    timestamp: &Instant,
    start: &Instant,
    duration: &Duration,
    refresh: &Duration,
    spinner: char,
) -> Result<String> {
    let dur = duration.num_milliseconds();
    let msg = if dur <= 3000 {
        format!("=> {} running [{}]", timestamp, spinner)
    } else {
        let head = format!("=> {} running ", timestamp);
        let tail = format!(" [{}]", spinner);
        let barsize = {
            let b = (dur / refresh.num_milliseconds()) as usize;
            let overhead = head.len() + tail.len() + 1;
            if b + overhead > width {
                width - overhead
            } else {
                b
            }
        };
        let elapsed = start.elapsed();
        let ratio = elapsed.num_milliseconds() as f32 / dur as f32;
        let left = if ratio < 1_f32 {
            ((barsize as f32) * ratio).ceil() as usize
        } else {
            barsize
        };
        let right = barsize.saturating_sub(left);
        let marker = if right == 0 { "=" } else { ">" };
        format!(
            "{}[{:=>left$}{:right$}]{}",
            head,
            marker,
            "",
            tail,
            left = left,
            right = right
        )
    };
    Ok(msg)
}

// Progbar object:

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Mode {
    None,
    Running,
    Sleeping,
}

#[derive(Debug)]
pub struct Progbar {
    mode: Mode,
    mode_wanted: Mode,
    shown: bool,
    start: Instant,
    duration: Duration,
    refresh_delay: Duration,
    ispinner: usize,
    lastrun: Instant,
    term: Term,
}

impl Default for Progbar {
    fn default() -> Progbar {
        Progbar {
            mode: Mode::None,
            mode_wanted: Mode::None,
            shown: false,
            start: Instant::now(),
            duration: Duration::default(),
            refresh_delay: Duration::milliseconds(250),
            ispinner: 0,
            lastrun: Instant::now(),
            term: Term::stdout(),
        }
    }
}

impl Progbar {
    pub fn set_running(&mut self, duration: Duration) {
        self.mode_wanted = Mode::Running;
        self.duration = duration;
        self.start = Instant::now();
    }

    pub fn set_sleep(&mut self, duration: Duration) {
        self.mode_wanted = Mode::Sleeping;
        self.duration = duration;
        self.start = Instant::now();
    }

    fn width(&self) -> usize {
        if let Some((_, w)) = self.term.size_checked() {
            w as usize
        } else {
            80
        }
    }

    fn spinner(&mut self) -> char {
        self.ispinner = (self.ispinner + 1) % 4;
        SPINNERS[self.ispinner]
    }

    pub fn hide(&mut self) -> Result<()> {
        if self.shown {
            self.term.move_cursor_up(1)?;
            self.term.clear_line()?;
            self.shown = false;
        }
        Ok(())
    }

    pub fn show(&mut self) -> Result<()> {
        self.refresh()
    }

    pub fn refresh(&mut self) -> Result<()> {
        if self.mode != self.mode_wanted {
            self.mode = self.mode_wanted;
        }
        self.hide()?;
        match self.mode {
            Mode::None => {
                return Ok(());
            }
            Mode::Sleeping => {
                let msg = progbar_sleeping(&self.lastrun, &self.start, &self.duration)?;
                self.term.write_line(&msg).map_err(Report::new)?;
            }
            Mode::Running => {
                let spinner = self.spinner();
                let msg = progbar_running(
                    self.width(),
                    &self.lastrun,
                    &self.start,
                    &self.duration,
                    &self.refresh_delay,
                    spinner,
                )?;
                self.term.write_line(&msg).map_err(Report::new)?;
            }
        }
        self.term.flush()?;
        self.shown = true;
        Ok(())
    }
}
