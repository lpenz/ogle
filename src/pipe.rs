// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use pin_project::pin_project;
use std::collections::VecDeque;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_stream::Stream;

use crate::differ::Differ;
use crate::input_stream::InputData;
use crate::input_stream::InputItem;
use crate::input_stream::InputStream;
use crate::output_sink::OutputCommand;
use crate::output_sink::WriteAll;
use crate::sys_input::Cmd;
use crate::sys_input::SysInputApi;

#[pin_project(project = PipeProjection)]
pub struct Pipe<SI: SysInputApi> {
    input_stream: InputStream<SI>,
    cmd: Cmd,
    pending: VecDeque<OutputCommand>,
    differ: Differ,
}

impl<SI: SysInputApi> Pipe<SI> {
    pub fn new(cmd: Cmd, input_stream: InputStream<SI>) -> Self {
        Pipe {
            input_stream,
            cmd,
            pending: VecDeque::default(),
            differ: Differ::default(),
        }
    }
}

impl<SI: SysInputApi> PipeProjection<'_, SI> {
    fn outline(&mut self, mut s: String) {
        s.push('\n');
        self.pending
            .push_back(OutputCommand::WriteAll(WriteAll(s.as_bytes().to_vec())))
    }
}

impl<SI: SysInputApi> Stream for Pipe<SI> {
    type Item = OutputCommand;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.as_mut().project();
        if let Some(output) = this.pending.pop_front() {
            return Poll::Ready(Some(output));
        }
        let item = Pin::new(&mut this.input_stream).poll_next(cx);
        match item {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Some(InputItem { time, data })) => match data {
                InputData::Start => {
                    this.outline(ofmt!(&time, "start execution"));
                    this.outline(format!("+ {}", this.cmd));
                    this.differ.reset();
                    self.poll_next(cx)
                }
                InputData::LineOut(line) => {
                    this.differ.push(line);
                    let mut differ = std::mem::take(this.differ);
                    if differ.has_changed() {
                        for line in &mut differ {
                            this.outline(line);
                        }
                    }
                    *this.differ = differ;
                    self.poll_next(cx)
                }
                InputData::LineErr(line) => {
                    this.differ.push(line);
                    let mut differ = std::mem::take(this.differ);
                    if differ.has_changed() {
                        for line in &mut differ {
                            this.outline(line);
                        }
                    }
                    *this.differ = differ;
                    self.poll_next(cx)
                }
                InputData::Done(sts) => {
                    this.outline(ofmt!(&time, "done {:?}", sts));
                    self.poll_next(cx)
                }
                InputData::Err(e) => {
                    this.outline(ofmt!(&time, "err {:?}", e));
                    self.poll_next(cx)
                }
                InputData::RunTick => {
                    this.outline(ofmt!(&time, "run   tick"));
                    self.poll_next(cx)
                }
                InputData::SleepTick => {
                    this.outline(ofmt!(&time, "sleep tick"));
                    self.poll_next(cx)
                }
            },
            Poll::Ready(None) => Poll::Ready(None),
        }
    }
}
