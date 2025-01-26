// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use color_eyre::Report;
use color_eyre::Result;
use console::Term;

use crate::misc::term_clear_line;
use crate::misc::term_width;
use crate::timewrap::Duration;
use crate::timewrap::Instant;

const SPINNERS: [char; 4] = ['/', '-', '\\', '|'];

// Basic functions:

pub fn progbar_sleeping(
    timestamp: &Instant,
    now: &Instant,
    start: &Instant,
    duration: &Duration,
) -> Result<String> {
    let msg = if duration.num_seconds() > 1 {
        let end = start + duration;
        let left = &end - now;
        format!("=> {} sleeping for {}s", timestamp, left.num_seconds() + 1)
    } else {
        format!("=> {} sleeping", timestamp)
    };
    Ok(msg)
}

pub fn progbar_running(
    width: usize,
    timestamp: &Instant,
    now: &Instant,
    start: &Instant,
    duration: &Option<Duration>,
    refresh: &Duration,
    spinner: char,
) -> Result<String> {
    let duration = duration.unwrap_or_default();
    let dur = duration.num_milliseconds();
    let msg = if dur <= 3000 {
        format!("=> {} running [{}]", timestamp, spinner)
    } else {
        let head = format!("=> {} running ", timestamp);
        let tail = format!(" [{}]", spinner);
        let barsize = {
            let b = (dur / refresh.num_milliseconds()) as usize;
            let overhead = head.len() + tail.len() + 1;
            debug_assert!(
                width >= overhead,
                "width {} not greater than overhead {}",
                width,
                overhead
            );
            if b + overhead > width {
                width - overhead
            } else {
                b
            }
        };
        let elapsed = now - start;
        let ratio = elapsed.num_milliseconds() as f32 / dur as f32;
        let left = if ratio < 1_f32 {
            ((barsize as f32) * ratio).ceil() as usize
        } else {
            barsize
        };
        let right = barsize.saturating_sub(left);
        let marker = if elapsed > duration { "=" } else { ">" };
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

    fn spinner(&mut self) -> char {
        self.ispinner = (self.ispinner + 1) % 4;
        SPINNERS[self.ispinner]
    }

    pub fn hide(&mut self) -> Result<()> {
        if self.shown {
            term_clear_line(&self.term)?;
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
                let msg =
                    progbar_sleeping(&self.lastrun, &Instant::now(), &self.start, &self.duration)?;
                self.term.write_line(&msg).map_err(Report::new)?;
            }
            Mode::Running => {
                let spinner = self.spinner();
                let msg = progbar_running(
                    term_width(&self.term),
                    &self.lastrun,
                    &Instant::now(),
                    &self.start,
                    &Some(self.duration),
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sleeping() {
        let start = Instant::epoch();
        let now = &start + &Duration::milliseconds(1);
        let string = progbar_sleeping(&now, &now, &start, &Duration::seconds(1)).unwrap();
        assert_eq!(string, "=> 1970-01-01 00:00:00 sleeping");
        let string = progbar_sleeping(&now, &now, &start, &Duration::seconds(10)).unwrap();
        assert_eq!(string, "=> 1970-01-01 00:00:00 sleeping for 10s");
        let now = &start + &Duration::seconds(10);
        let string = progbar_sleeping(&now, &now, &start, &Duration::seconds(10)).unwrap();
        assert_eq!(string, "=> 1970-01-01 00:00:10 sleeping for 1s");
    }

    #[test]
    fn running_fast() {
        let start = Instant::epoch();
        let now = &start + &Duration::milliseconds(1);
        let string = progbar_running(
            80,
            &now,
            &now,
            &start,
            &Some(Duration::seconds(3)),
            &Duration::default(),
            'X',
        )
        .unwrap();
        assert_eq!(string, "=> 1970-01-01 00:00:00 running [X]");
    }

    #[test]
    fn running_shortbar() {
        let start = Instant::epoch();
        let f = |n| {
            let now = &start + &Duration::milliseconds(1);
            let now = &now + &Duration::seconds(n);
            progbar_running(
                80,
                &start,
                &now,
                &start,
                &Some(Duration::seconds(4)),
                &Duration::seconds(1),
                'X',
            )
            .unwrap()
        };
        let string = f(0);
        assert_eq!(string, "=> 1970-01-01 00:00:00 running [>   ] [X]");
        let string = f(1);
        assert_eq!(string, "=> 1970-01-01 00:00:00 running [=>  ] [X]");
        let string = f(2);
        assert_eq!(string, "=> 1970-01-01 00:00:00 running [==> ] [X]");
        let string = f(3);
        assert_eq!(string, "=> 1970-01-01 00:00:00 running [===>] [X]");
        let string = f(4);
        assert_eq!(string, "=> 1970-01-01 00:00:00 running [====] [X]");
    }

    #[test]
    fn running_longbar() {
        let start = Instant::epoch();
        let f = |n| {
            let now = &start + &Duration::milliseconds(1);
            let now = &now + &Duration::seconds(n);
            progbar_running(
                40,
                &start,
                &now,
                &start,
                &Some(Duration::seconds(40)),
                &Duration::seconds(1),
                'X',
            )
            .unwrap()
        };
        let string = f(0);
        assert_eq!(string, "=> 1970-01-01 00:00:00 running [>   ] [X]");
        let string = f(10);
        assert_eq!(string, "=> 1970-01-01 00:00:00 running [=>  ] [X]");
        let string = f(20);
        assert_eq!(string, "=> 1970-01-01 00:00:00 running [==> ] [X]");
        let string = f(30);
        assert_eq!(string, "=> 1970-01-01 00:00:00 running [===>] [X]");
        let string = f(40);
        assert_eq!(string, "=> 1970-01-01 00:00:00 running [====] [X]");
    }
}
