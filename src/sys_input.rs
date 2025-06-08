// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Module that wraps system functions used as inputs
//!
//! Wrapping this makes it very easy to test the whole program.

use color_eyre::Result;
use pin_project::pin_project;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt;
use std::io;
use std::pin::Pin;
use std::process::ExitStatus;
use std::process::Stdio;
use std::task::{Context, Poll};
use tokio::process::Command;
use tokio_process_stream as tps;
use tokio_stream::Stream;
use tracing::instrument;

use crate::term_wrapper;
use crate::time_wrapper::Duration;
use crate::time_wrapper::Instant;

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

impl From<&[&str]> for Cmd {
    fn from(s: &[&str]) -> Cmd {
        Self(s.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }
}

impl fmt::Display for Cmd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let joined = self.0.join(" ");
        write!(f, "{}", joined)
    }
}

//////////////////////////////////////////////////////////////////////////////

/// A clonable, PartialEq replacement for [`tokio_process_stream::Item`]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    /// A stdout line printed by the process.
    Stdout(String),
    /// A stderr line printed by the process.
    Stderr(String),
    /// The [`ExitStatus`](std::process::ExitStatus), yielded after the process exits.
    Done(Result<ExitStatus, io::ErrorKind>),
}

impl From<tps::Item<String>> for Item {
    fn from(item: tps::Item<String>) -> Self {
        match item {
            tps::Item::Stdout(s) => Item::Stdout(s),
            tps::Item::Stderr(s) => Item::Stderr(s),
            tps::Item::Done(result) => Item::Done(result.map_err(|e| e.kind())),
        }
    }
}

/// A mockable wrapper for [`tokio_process_stream::ProcessLineStream`].
#[pin_project(project = ProcessStreamProj)]
pub enum ProcessStream {
    /// Wrapper for [`tokio_process_stream::ProcessLineStream`].
    Real { stream: Box<tps::ProcessLineStream> },
    /// Mock for a running process stream that just returns items from
    /// a list. Useful for testing.
    Virtual { items: VecDeque<Item> },
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

//////////////////////////////////////////////////////////////////////////////

/// Wrap the system functions we use as inputs.
///
/// This wrapper makes testing easy.
pub trait SysInputApi: std::fmt::Debug + Clone + Default {
    fn now(&self) -> Instant;
    #[allow(dead_code)]
    fn size_checked(&self) -> Option<(u16, u16)>;
    fn run_command(&mut self, command: Cmd) -> Result<ProcessStream, std::io::Error>;
}

/// [`SysInputApi`] implementation of the real environment.
#[derive(Debug, Clone, Default)]
pub struct SysInputReal {}

impl SysInputApi for SysInputReal {
    fn now(&self) -> Instant {
        Instant::from(chrono::offset::Utc::now())
    }
    fn size_checked(&self) -> Option<(u16, u16)> {
        term_wrapper::size_checked()
    }
    fn run_command(&mut self, cmd: Cmd) -> Result<ProcessStream, std::io::Error> {
        let process_stream = tps::ProcessLineStream::try_from(cmd.get_command())?;
        Ok(ProcessStream::from(process_stream))
    }
}

/// [`SysInputApi`] implementation of a virtual environment, to be used in tests.
#[derive(Debug, Clone, Default)]
pub struct SysInputVirtual {
    now: RefCell<Instant>,
    items: VecDeque<Item>,
}

impl SysInputApi for SysInputVirtual {
    fn now(&self) -> Instant {
        let mut now_ref = self.now.borrow_mut();
        let now = *now_ref;
        *now_ref = &now + &Duration::seconds(1);
        now
    }
    fn size_checked(&self) -> Option<(u16, u16)> {
        Some((25, 80))
    }
    fn run_command(&mut self, _cmd: Cmd) -> Result<ProcessStream, std::io::Error> {
        let items = std::mem::take(&mut self.items);
        Ok(ProcessStream::from(items))
    }
}

#[cfg(test)]
impl SysInputVirtual {
    pub fn set_items(&mut self, items: Vec<Item>) {
        self.items = items.into_iter().collect();
    }
}

#[cfg(test)]
pub mod test {
    use color_eyre::Result;
    use color_eyre::eyre::eyre;
    use tokio_stream::StreamExt;

    use crate::sys_input::SysInputReal;
    use crate::sys_input::SysInputVirtual;

    use super::*;

    // Tests for SysInputReal with simple unix bins as we don't cover
    // it in downstream tests

    async fn stream_cmd(
        cmdstr: &[&str],
    ) -> Result<impl StreamExt<Item = Item> + std::marker::Unpin + Send + 'static> {
        let cmd = Cmd::from(cmdstr);
        let mut sys = SysInputReal::default();
        sys.run_command(cmd).map_err(|e| eyre!(e))
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

    #[test]
    fn test_now() {
        let sys = SysInputReal::default();
        let now = sys.now();
        let now2 = sys.now();
        assert!(&now2 >= &now);
    }

    // A simple test for SysInputVirtual as we cover it better in
    // downstream tests

    #[tokio::test]
    async fn test_sysinputvirtual() -> Result<()> {
        let list = vec![
            Item::Stdout("stdout".into()),
            Item::Stderr("stderr".into()),
            Item::Done(Ok(ExitStatus::default())),
        ];
        let mut sys = SysInputVirtual::default();
        sys.set_items(list.clone());
        let cmd = Cmd::default();
        assert_eq!(format!("{}", cmd), "");
        let streamer = sys.run_command(cmd)?;
        assert_eq!(format!("{:?}", streamer), "ProcessStream::Virtual");
        let streamed = streamer.collect::<Vec<_>>().await;
        assert_eq!(streamed, list);
        assert_eq!(sys.now(), Instant::default());
        assert_eq!(sys.now(), &Instant::default() + &Duration::seconds(1));
        assert_eq!(sys.size_checked(), Some((25, 80)));
        Ok(())
    }
}
