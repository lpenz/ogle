// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Module that wraps system functions used as inputs
//!
//! Wrapping this makes it very easy to test the whole program.

use color_eyre::Result;
use pin_project_lite::pin_project;
use std::collections::VecDeque;
use std::pin::Pin;
use std::process::Stdio;
use std::task::{Context, Poll};
use tokio::process::Command;
use tokio_process_stream as tps;
use tokio_stream::Stream;

//////////////////////////////////////////////////////////////////////////////

/// A [`tokio::process::Command`] pseudo-wrapper that `impl Clone`.
#[derive(Debug, Default, Clone)]
pub struct Cmd(Vec<String>);

impl Cmd {
    pub fn get_command(self) -> Command {
        let mut command = Command::new(&self.0[0]);
        command.args(self.0.iter().skip(1));
        command.stdin(Stdio::null());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        command
    }
}

impl From<Vec<String>> for Cmd {
    fn from(s: Vec<String>) -> Cmd {
        Self(s)
    }
}

//////////////////////////////////////////////////////////////////////////////

pub type Item = tps::Item<String>;

pin_project! {
#[project = ProcessStreamProj]
pub enum ProcessStream {
    Real { stream: tps::ProcessLineStream},
    Virtual { items: VecDeque<Item> },
}
}

impl From<tps::ProcessLineStream> for ProcessStream {
    fn from(stream: tps::ProcessLineStream) -> Self {
        ProcessStream::Real { stream }
    }
}

impl From<VecDeque<Item>> for ProcessStream {
    fn from(items: VecDeque<Item>) -> Self {
        ProcessStream::Virtual { items }
    }
}

impl Stream for ProcessStream {
    type Item = Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match this {
            ProcessStreamProj::Real { stream } => Pin::new(stream).poll_next(cx),
            ProcessStreamProj::Virtual { items } => {
                if let Some(item) = items.pop_front() {
                    Poll::Ready(Some(item))
                } else {
                    Poll::Ready(None)
                }
            }
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

/// Wrap the system functions we use to build the [`InputStream`].
///
/// This wrapper makes testing easy.
pub trait SysInputApi: std::fmt::Debug + Clone + Default {
    fn run_command(&mut self, command: Cmd) -> Result<ProcessStream, std::io::Error>;
}

/// [`SysInputApi`] implementation of the real environment.
#[derive(Debug, Clone, Default)]
pub struct SysInputReal {}

impl SysInputApi for SysInputReal {
    fn run_command(&mut self, cmd: Cmd) -> Result<ProcessStream, std::io::Error> {
        let process_stream = tps::ProcessLineStream::try_from(cmd.get_command())?;
        Ok(ProcessStream::from(process_stream))
    }
}

/// [`SysInputApi`] implementation of a virtual environment, to be used in tests.
#[derive(Debug, Default)]
pub struct SysInputVirtual {
    items: VecDeque<Item>,
}

impl Clone for SysInputVirtual {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl SysInputApi for SysInputVirtual {
    fn run_command(&mut self, _cmd: Cmd) -> Result<ProcessStream, std::io::Error> {
        let items = std::mem::take(&mut self.items);
        Ok(ProcessStream::from(items))
    }
}

impl SysInputVirtual {
    #[cfg(test)]
    pub fn set_items(&mut self, items: Vec<Item>) {
        self.items = items.into_iter().collect();
    }
}

#[cfg(test)]
pub mod test {}

//////////////////////////////////////////////////////////////////////////////
