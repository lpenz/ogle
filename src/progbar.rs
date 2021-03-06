// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use anyhow::Result;
use console::Term;
use tokio::time;

const SPINNERS: [char; 4] = ['/', '-', '\\', '|'];

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
    start: time::Instant,
    duration: time::Duration,
    refresh_delay: time::Duration,
    ispinner: usize,
    term: Term,
}

impl Default for Progbar {
    fn default() -> Progbar {
        Progbar {
            mode: Mode::None,
            mode_wanted: Mode::None,
            shown: false,
            start: time::Instant::now(),
            duration: time::Duration::from_secs(0),
            refresh_delay: time::Duration::from_millis(250),
            ispinner: 0,
            term: Term::stdout(),
        }
    }
}

impl Progbar {
    pub fn set_running(&mut self, duration: time::Duration) {
        self.mode_wanted = Mode::Running;
        self.duration = duration;
        self.start = time::Instant::now();
    }

    pub fn set_sleep(&mut self, duration: time::Duration) {
        self.mode_wanted = Mode::Sleeping;
        self.duration = duration;
        self.start = time::Instant::now();
    }

    fn width(&self) -> usize {
        if let Some((_, w)) = self.term.size_checked() {
            w as usize
        } else {
            80
        }
    }

    fn barsize(&self, overhead: usize, dur: u128, refresh: u128) -> usize {
        let width = self.width();
        let barsize = (dur / refresh) as usize;
        if barsize + overhead > width {
            width - overhead
        } else {
            barsize
        }
    }

    fn proginfo(&self, overhead: usize) -> (usize, usize, usize) {
        let dur = self.duration.as_millis();
        let refresh = self.refresh_delay.as_millis();
        let total = self.barsize(overhead, dur, refresh);
        let elapsed = self.elapsed();
        let ratio = elapsed.as_millis() as f32 / self.duration.as_millis() as f32;
        let left = if ratio < 1_f32 {
            ((total as f32) * ratio).ceil() as usize
        } else {
            total
        };
        let right = if left < total { total - left } else { 0 };
        (left, right, total)
    }

    fn elapsed(&self) -> time::Duration {
        time::Instant::now() - self.start
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
                let msg = if self.duration.as_secs() > 1 {
                    let end = self.start + self.duration;
                    let left = end - time::Instant::now();
                    format!("=> sleeping for {}s", left.as_secs() + 1)
                } else {
                    "=> sleeping".to_string()
                };
                self.term.write_line(&msg)?;
            }
            Mode::Running => {
                let dur = self.duration.as_millis();
                let msg = if dur <= 3000 {
                    format!("=> running [{}]", self.spinner())
                } else {
                    let header = "=> running ";
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
