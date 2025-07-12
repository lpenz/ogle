// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Wrapper for process functions.
//!
//! # `Cmd`
//!
//! The [`Cmd`] type has an inner `Vec<String>` that we can turn into
//! a [`tokio::process::Command`]. It implements `Clone`, which we use
//! to spawn the same process multiple times.
//!
//! # `ProcessStream`
//!
//! The [`ProcessStream`] type wraps [`tokio_process_stream`] in order
//! to provide an [`Item`] that implements `Eq` which we can then use
//! for testing.

use color_eyre::Result;
use pin_project::pin_project;
use std::collections::VecDeque;
use std::fmt;
use std::io;
use std::os::unix::process::ExitStatusExt;
use std::pin::Pin;
use std::process::ExitStatus;
use std::process::Stdio;
use std::task::{Context, Poll};
use tokio::process::Child;
use tokio::process::Command;
use tokio_process_stream as tps;
use tokio_stream::Stream;
use tracing::instrument;

// Command wrapper ///////////////////////////////////////////////////

/// A [`tokio::process::Command`] pseudo-wrapper that `impl Clone`.
#[derive(Debug, Default, Clone)]
pub struct Cmd(Vec<String>);

impl From<&Cmd> for Command {
    fn from(cmd: &Cmd) -> Command {
        let mut command = Command::new(&cmd.0[0]);
        command.args(cmd.0.iter().skip(1));
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

impl From<&[&str]> for Cmd {
    fn from(s: &[&str]) -> Cmd {
        Self(s.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }
}

impl fmt::Display for Cmd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let joined = self.0.join(" ");
        write!(f, "{joined}")
    }
}

// Exit status ///////////////////////////////////////////////////////

/// A [`std::process::ExitStatus`] pseudo-wrapper that impl Clone
/// and custom Display.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum ExitSts {
    #[default]
    Success,
    Code(u8),
    Signal(i32),
}

impl ExitSts {
    pub fn success(&self) -> bool {
        self == &ExitSts::Success
    }
}

impl From<ExitStatus> for ExitSts {
    fn from(sts: ExitStatus) -> ExitSts {
        if sts.success() {
            ExitSts::Success
        } else if let Some(code) = sts.code() {
            ExitSts::Code(code as u8)
        } else if let Some(signal) = sts.signal() {
            ExitSts::Signal(signal)
        } else {
            panic!("Unable to figure out exit status {sts:?}")
        }
    }
}

impl fmt::Display for ExitSts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExitSts::Success => write!(f, "success"),
            ExitSts::Code(code) => write!(f, "code {code}"),
            ExitSts::Signal(signal) => write!(f, "signal {signal}"),
        }
    }
}

// ProcessStream /////////////////////////////////////////////////////

/// A clonable, Eq replacement for [`tokio_process_stream::Item`]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    /// A stdout line printed by the process.
    Stdout(String),
    /// A stderr line printed by the process.
    Stderr(String),
    /// The [`ExitSts`], yielded after the process exits.
    Done(Result<ExitSts, io::ErrorKind>),
}

impl From<tps::Item<String>> for Item {
    fn from(item: tps::Item<String>) -> Self {
        match item {
            tps::Item::Stdout(s) => Item::Stdout(s),
            tps::Item::Stderr(s) => Item::Stderr(s),
            tps::Item::Done(result) => Item::Done(match result {
                Ok(sts) => Ok(sts.into()),
                Err(err) => Err(err.kind()),
            }),
        }
    }
}

/// A wrapper for [`tokio_process_stream::ProcessLineStream`].
///
/// Also provides a virtual implementation for use in tests.
#[pin_project(project = ProcessStreamProj)]
pub enum ProcessStream {
    /// Wrapper for [`tokio_process_stream::ProcessLineStream`].
    Real { stream: Box<tps::ProcessLineStream> },
    /// Mock for a running process stream that just returns items from
    /// a list. Useful for testing.
    Virtual { items: VecDeque<Item> },
}

impl ProcessStream {
    /// Return a mutable reference to the child object
    pub fn child_mut(&mut self) -> Option<&mut Child> {
        if let ProcessStream::Real { stream } = self {
            stream.child_mut()
        } else {
            None
        }
    }
}

impl std::fmt::Debug for ProcessStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessStream::Real { stream: _ } => f.debug_struct("ProcessStream::Real"),
            ProcessStream::Virtual { items: _ } => f.debug_struct("ProcessStream::Virtual"),
        }
        .finish()
    }
}

impl From<tps::ProcessLineStream> for ProcessStream {
    fn from(stream: tps::ProcessLineStream) -> Self {
        ProcessStream::Real {
            stream: Box::new(stream),
        }
    }
}

impl From<VecDeque<Item>> for ProcessStream {
    fn from(items: VecDeque<Item>) -> Self {
        ProcessStream::Virtual { items }
    }
}

impl Stream for ProcessStream {
    type Item = Item;

    #[instrument(level = "debug", ret, skip(cx))]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match this {
            ProcessStreamProj::Real { stream } => {
                let next = Pin::new(stream).poll_next(cx);
                match next {
                    Poll::Ready(opt) => Poll::Ready(opt.map(|i| i.into())),
                    Poll::Pending => Poll::Pending,
                }
            }
            ProcessStreamProj::Virtual { items } => Poll::Ready(items.pop_front()),
        }
    }
}

// Tests /////////////////////////////////////////////////////////////

#[cfg(test)]
pub mod test {
    use color_eyre::Result;
    use color_eyre::eyre::eyre;
    use tokio_stream::StreamExt;

    use super::*;

    async fn stream_cmd(
        cmdstr: &[&str],
    ) -> Result<impl StreamExt<Item = Item> + std::marker::Unpin + Send + 'static> {
        let cmd = Cmd::from(cmdstr);
        let process_stream = tps::ProcessLineStream::try_from(Command::from(&cmd))?;
        Ok(ProcessStream::from(process_stream))
    }

    async fn stream_next<T>(stream: &mut T) -> Result<Item>
    where
        T: StreamExt<Item = Item> + std::marker::Unpin + Send + 'static,
    {
        stream.next().await.ok_or(eyre!("no item received"))
    }

    #[tokio::test]
    async fn test_true() -> Result<()> {
        let mut stream = stream_cmd(&["true"]).await?;
        let item = stream_next(&mut stream).await?;
        let Item::Done(sts) = item else {
            return Err(eyre!("unexpected stream item {:?}", item));
        };
        assert!(sts.unwrap().success());
        assert!(stream.next().await.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn test_false() -> Result<()> {
        let mut stream = stream_cmd(&["false"]).await?;
        let item = stream_next(&mut stream).await?;
        let Item::Done(sts) = item else {
            return Err(eyre!("unexpected stream item {:?}", item));
        };
        assert!(!sts.unwrap().success());
        Ok(())
    }

    #[tokio::test]
    async fn test_echo() -> Result<()> {
        let mut stream = stream_cmd(&["echo", "test"]).await?;
        let item = stream_next(&mut stream).await?;
        let Item::Stdout(s) = item else {
            return Err(eyre!("unexpected stream item {:?}", item));
        };
        assert_eq!(s, "test");
        let item = stream_next(&mut stream).await?;
        let Item::Done(sts) = item else {
            return Err(eyre!("unexpected stream item {:?}", item));
        };
        assert!(sts.unwrap().success());
        Ok(())
    }

    #[tokio::test]
    async fn test_stderr() -> Result<()> {
        let mut stream = stream_cmd(&["/bin/sh", "-c", "echo test >&2"]).await?;
        let item = stream_next(&mut stream).await?;
        let Item::Stderr(s) = item else {
            return Err(eyre!("unexpected stream item {:?}", item));
        };
        assert_eq!(s, "test");
        let item = stream_next(&mut stream).await?;
        let Item::Done(sts) = item else {
            return Err(eyre!("unexpected stream item {:?}", item));
        };
        assert!(sts.unwrap().success());
        Ok(())
    }
}
