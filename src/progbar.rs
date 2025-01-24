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
    term: &Term,
    timestamp: &Instant,
    start: &Instant,
    duration: &Duration,
) -> Result<()> {
    let msg = if duration.num_seconds() > 1 {
        let end = start + duration;
        let left = &end - &Instant::now();
        format!("=> {} sleeping for {}s", timestamp, left.num_seconds() + 1)
    } else {
        format!("=> {} sleeping", timestamp)
    };
    term.write_line(&msg).map_err(Report::new)
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

    fn barsize(&self, overhead: usize, dur: i64, refresh: i64) -> usize {
        let width = self.width();
        let barsize = (dur / refresh) as usize;
        if barsize + overhead > width {
            width - overhead
        } else {
            barsize
        }
    }

    fn proginfo(&self, overhead: usize) -> (usize, usize, usize) {
        let dur = self.duration.num_milliseconds();
        let refresh = self.refresh_delay.num_milliseconds();
        let total = self.barsize(overhead, dur, refresh);
        let elapsed = self.start.elapsed();
        let ratio = elapsed.num_milliseconds() as f32 / self.duration.num_milliseconds() as f32;
        let left = if ratio < 1_f32 {
            ((total as f32) * ratio).ceil() as usize
        } else {
            total
        };
        let right = total.saturating_sub(left);
        (left, right, total)
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
                progbar_sleeping(&self.term, &self.lastrun, &self.start, &self.duration)?;
            }
            Mode::Running => {
                let dur = self.duration.num_milliseconds();
                self.lastrun = Instant::now();
                let msg = if dur <= 3000 {
                    let spinner = self.spinner();
                    format!("=> {} running [{}]", self.lastrun, spinner)
                } else {
                    let header = format!("=> {} running ", self.lastrun);
                    let (left, right, _) = self.proginfo(header.len() + 6);
                    let marker = if right == 0 { "=" } else { ">" };
                    format!(
                        "{}[{:=>left$}{:right$}] [{}]",
                        header,
                        marker,
                        "",
                        self.spinner(),
                        left = left,
                        right = right
                    )
                };
                self.term.write_line(&msg)?;
            }
        }
        self.term.flush()?;
        self.shown = true;
        Ok(())
    }
}
