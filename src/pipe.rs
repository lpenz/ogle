// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_stream::Stream;

use crate::input_stream::InputData;
use crate::input_stream::InputItem;
use crate::input_stream::InputStream;
use crate::output_sink::OutputCommand;
use crate::output_sink::WriteAll;
use crate::sys_input::SysInputApi;

pin_project! {
#[derive(Default)]
pub struct Pipe<SI: SysInputApi> {
    input_stream: InputStream<SI>,
}
}

impl<SI: SysInputApi> Pipe<SI> {}

impl<SI: SysInputApi> From<InputStream<SI>> for Pipe<SI> {
    fn from(input_stream: InputStream<SI>) -> Pipe<SI> {
        Pipe { input_stream }
    }
}

fn writeline(mut s: String) -> OutputCommand {
    s.push('\n');
    OutputCommand::WriteAll(WriteAll(s.as_bytes().to_vec()))
}

impl<SI: SysInputApi> Stream for Pipe<SI> {
    type Item = OutputCommand;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match Pin::new(this.input_stream).poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Some(InputItem { time: _, data })) => match data {
                InputData::Start => Poll::Pending,
                InputData::LineOut(line) => Poll::Ready(Some(writeline(line))),
                InputData::LineErr(line) => Poll::Ready(Some(writeline(line))),
                InputData::Done(sts) => Poll::Ready(Some(writeline(format!("done {:?}", sts)))),
                InputData::Err(e) => Poll::Ready(Some(writeline(format!("err {:?}", e)))),
                InputData::Tick => Poll::Pending,
            },
            Poll::Ready(None) => Poll::Pending,
        }
    }
}
