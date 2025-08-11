// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use pin_project::pin_project;
use std::collections::VecDeque;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_stream::Stream;

use crate::differ::Differ;
use crate::engine::EData;
use crate::engine::EItem;
use crate::engine::Engine;
use crate::output::ClearLine;
use crate::output::MoveCursorUp;
use crate::output::OutputCommand;
use crate::output::WriteAll;
use crate::process_wrapper::Cmd;
use crate::progbar::progbar_running;
use crate::progbar::progbar_sleeping;
use crate::progbar::spinner_get;
use crate::sys::SysApi;
use crate::time_wrapper::Duration;
use crate::time_wrapper::Instant;

#[pin_project(project = ViewProjection)]
pub struct View<SI: SysApi> {
    // Configuration parameters:
    cmd: Cmd,
    refresh: Duration,
    sleep: Duration,
    /// The engine that streams the all events.
    engine: Engine<SI>,
    /// Some engine events generate more than one item; store them
    /// here and yield them in the next calls.
    pending: VecDeque<OutputCommand>,
    /// The differ that stores the lines so that we can compare runs.
    differ: Differ,
    /// Spinner state
    spinner: char,
    start: Instant, // can be start of running or sleep
    duration: Option<Duration>,
    /// If the status is currently visible and has to be cleared.
    printed_status: bool,
    /// Number of runs so far.
    runs: u32,
    /// Approximate time when we'll wake up with the next StartRun,
    /// used for the countdown.
    sleep_deadline: Option<Instant>,
}

impl<SI: SysApi> View<SI> {
    pub fn new(cmd: Cmd, refresh: Duration, sleep: Duration, engine: Engine<SI>) -> Self {
        View {
            cmd,
            refresh,
            sleep,
            engine,
            pending: VecDeque::default(),
            differ: Differ::default(),
            spinner: '-',
            start: Instant::default(),
            duration: None,
            printed_status: false,
            runs: 0,
            sleep_deadline: None,
        }
    }
}

impl<SI: SysApi> ViewProjection<'_, SI> {
    fn _println(&mut self, mut s: String) {
        s.push('\n');
        self.pending
            .push_back(OutputCommand::WriteAll(WriteAll(s.as_bytes().to_vec())))
    }

    fn status_maybe_clear(&mut self) {
        if *self.printed_status {
            self.pending
                .push_back(OutputCommand::MoveCursorUp(MoveCursorUp(1)));
            self.pending
                .push_back(OutputCommand::ClearLine(ClearLine {}));
        }
    }

    fn println(&mut self, s: String) {
        self.status_maybe_clear();
        self._println(s);
        *self.printed_status = false;
    }

    fn process_line(&mut self, line: String) {
        self.differ.push(line);
        let mut differ = std::mem::take(self.differ);
        if differ.has_changed() {
            for line in &mut differ {
                self.println(line);
            }
        }
        *self.differ = differ;
    }

    fn status_update_running(&mut self, now: Instant) {
        self.status_maybe_clear();
        let mut spinner = *self.spinner;
        self._println(ofmt!(
            &now,
            "{}",
            progbar_running(
                150,                       // width: usize,
                &now,                      // now: &Instant,
                self.start,                // start: &Instant,
                *self.duration,            // duration: Option<&Duration>,
                self.refresh,              // refresh: &Duration,
                spinner_get(&mut spinner)  // spinner: char,
            )
            .unwrap()
        ));
        *self.spinner = spinner;
        *self.printed_status = true;
    }

    fn status_update_sleeping(&mut self, now: Instant, deadline: Instant) {
        self.status_maybe_clear();
        let mut spinner = *self.spinner;
        self._println(ofmt!(
            self.start,
            "{}",
            progbar_sleeping(self.sleep, &now, &deadline, spinner_get(&mut spinner))
        ));
        *self.spinner = spinner;
        *self.printed_status = true;
    }
}

impl<SI: SysApi> Stream for View<SI> {
    type Item = OutputCommand;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.as_mut().project();
        if let Some(output) = this.pending.pop_front() {
            return Poll::Ready(Some(output));
        }
        let item = Pin::new(&mut this.engine).poll_next(cx);
        match item {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Some(EItem { time: now, data })) => match data {
                EData::StartRun => {
                    if *this.runs == 0 {
                        this.println(ofmt!(&now, "start execution"));
                    }
                    this.differ.reset();
                    *this.sleep_deadline = None;
                    *this.start = now;
                    this.status_update_running(now);
                    this.process_line(ofmt_timeless!("+ {}", this.cmd));
                    self.poll_next(cx)
                }
                EData::StartSleep(deadline) => {
                    self.sleep_deadline = Some(deadline);
                    self.poll_next(cx)
                }
                EData::LineOut(line) => {
                    this.process_line(line);
                    this.status_update_running(now);
                    self.poll_next(cx)
                }
                EData::LineErr(line) => {
                    this.process_line(line);
                    this.status_update_running(now);
                    self.poll_next(cx)
                }
                EData::Done(sts) => {
                    let line = ofmt_timeless!("exited with {}", sts);
                    this.process_line(line);
                    *this.duration = Some(&now - this.start);
                    // Sleeping starts now
                    *this.start = now;
                    *this.runs += 1;
                    self.poll_next(cx)
                }
                EData::Err(e) => {
                    this.println(ofmt!(&now, "err {:?}", e));
                    self.poll_next(cx)
                }
                EData::Tick => {
                    if let Some(deadline) = *this.sleep_deadline {
                        this.status_update_sleeping(now, deadline);
                    } else {
                        this.status_update_running(now);
                    }
                    self.poll_next(cx)
                }
            },
            Poll::Ready(None) => Poll::Ready(None),
        }
    }
}
