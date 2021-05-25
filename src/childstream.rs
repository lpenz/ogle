// Copyright (C) 2021 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use pin_project_lite::pin_project;
use std::convert;
use std::fmt;
use std::io;
use std::pin::Pin;
use std::process::ExitStatus;
use std::process::Stdio;
use std::task::Context;
use std::task::Poll;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::process::Child;
use tokio::process::Command;
use tokio::process::{ChildStderr, ChildStdout};
use tokio_stream::wrappers::LinesStream;
use tokio_stream::Stream;

#[derive(Debug, PartialEq, Eq)]
pub enum Item {
    Stdout(String),
    Stderr(String),
    Done(ExitStatus),
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Item::Stdout(s) => fmt::Display::fmt(&s, f),
            Item::Stderr(s) => fmt::Display::fmt(&s, f),
            _ => Ok(()),
        }
    }
}

pin_project! {
#[derive(Debug)]
pub struct ChildStream {
    pub child: Option<Child>,
    pub stdout: Option<LinesStream<BufReader<ChildStdout>>>,
    pub stderr: Option<LinesStream<BufReader<ChildStderr>>>,
}
}

impl convert::From<Child> for ChildStream {
    fn from(mut child: Child) -> ChildStream {
        let stdout = child
            .stdout
            .take()
            .map(|s| LinesStream::new(BufReader::new(s).lines()));
        let stderr = child
            .stderr
            .take()
            .map(|s| LinesStream::new(BufReader::new(s).lines()));
        Self {
            child: Some(child),
            stdout,
            stderr,
        }
    }
}

impl convert::TryFrom<Command> for ChildStream {
    type Error = io::Error;
    fn try_from(mut command: Command) -> io::Result<ChildStream> {
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        let child = command.spawn()?;
        Ok(Self::from(child))
    }
}

impl Stream for ChildStream {
    type Item = Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.child.is_none() {
            // Keep returning None after we are done and everything is dropped
            return Poll::Ready(None);
        }
        let this = self.project();
        if let Some(stderr) = this.stderr {
            match Pin::new(stderr).poll_next(cx) {
                Poll::Ready(Some(line)) => {
                    return Poll::Ready(Some(Item::Stderr(line.unwrap())));
                }
                Poll::Ready(None) => {
                    *this.stderr = None;
                }
                Poll::Pending => {}
            }
        }
        if let Some(stdout) = this.stdout {
            match Pin::new(stdout).poll_next(cx) {
                Poll::Ready(Some(line)) => {
                    return Poll::Ready(Some(Item::Stdout(line.unwrap())));
                }
                Poll::Ready(None) => {
                    *this.stdout = None;
                }
                Poll::Pending => {}
            }
        }
        if this.stdout.is_none() && this.stderr.is_none() {
            if let Some(ref mut child) = this.child {
                if let Ok(Some(sts)) = child.try_wait() {
                    *this.child = None;
                    return Poll::Ready(Some(Item::Done(sts)));
                }
            }
        }
        Poll::Pending
    }
}
