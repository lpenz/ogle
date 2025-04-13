// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use pin_project::pin_project;
use std::collections::VecDeque;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_stream::Stream;

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
}

impl<SI: SysInputApi> Pipe<SI> {
    pub fn new(cmd: Cmd, input_stream: InputStream<SI>) -> Self {
        Pipe {
            input_stream,
            cmd,
            pending: VecDeque::default(),
        }
    }
}

impl<SI: SysInputApi> PipeProjection<'_, SI> {
    fn outline(&mut self, mut s: String) {
        s.push('\n');
        self.pending
            .push_back(OutputCommand::WriteAll(WriteAll(s.as_bytes().to_vec())))
    }

    fn flush(&mut self) -> Poll<Option<OutputCommand>> {
        if let Some(output) = self.pending.pop_front() {
            Poll::Ready(Some(output))
        } else {
            Poll::Pending
        }
    }
}

impl<SI: SysInputApi> Stream for Pipe<SI> {
    type Item = OutputCommand;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        let item = Pin::new(&mut this.input_stream).poll_next(cx);
        match item {
            Poll::Pending => {}
            Poll::Ready(Some(InputItem { time, data })) => match data {
                InputData::Start => {
                    this.outline(ofmt!(&time, "start execution"));
                    this.outline(format!("+ {}", this.cmd));
                }
                InputData::LineOut(line) => {
                    this.outline(line);
                }
                InputData::LineErr(line) => {
                    this.outline(line);
                }
                InputData::Done(sts) => {
                    this.outline(ofmt!(&time, "done {:?}", sts));
                }
                InputData::Err(e) => {
                    this.outline(ofmt!(&time, "err {:?}", e));
                }
                InputData::Tick => {}
            },
            Poll::Ready(None) => {
                // We don't care if the input dried out, it's going to
                // run again soon anyway.
            }
        };
        this.flush()
    }
}
