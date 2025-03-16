// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use color_eyre::Result;
use pin_project_lite::pin_project;
use std::collections::VecDeque;
use std::pin::Pin;
use std::process::ExitStatus;
use std::task::{Context, Poll};
use tokio_process_stream as tps;
use tokio_stream::wrappers::IntervalStream;
use tokio_stream::Stream;

use crate::time_wrapper::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamItem {
    LineOut(String),
    LineErr(String),
    Done(ExitStatus),
    Err(std::io::ErrorKind),
    Tick,
}

impl From<tps::Item<String>> for StreamItem {
    fn from(item: tps::Item<String>) -> Self {
        match item {
            tps::Item::Stdout(l) => StreamItem::LineOut(l),
            tps::Item::Stderr(l) => StreamItem::LineErr(l),
            tps::Item::Done(Ok(sts)) => StreamItem::Done(sts),
            tps::Item::Done(Err(e)) => StreamItem::from(e),
        }
    }
}

impl From<tokio::time::Instant> for StreamItem {
    fn from(_: tokio::time::Instant) -> Self {
        StreamItem::Tick
    }
}

impl From<std::io::Error> for StreamItem {
    fn from(e: std::io::Error) -> Self {
        StreamItem::Err(e.kind())
    }
}

pin_project! {
#[derive(Debug, Default)]
pub struct Streamer {
    // Events coming from the running process
    process: Option<tps::ProcessLineStream>,
    // Tick events generated by the [`IntervalStream`] timer
    ticker: Option<IntervalStream>,
    // Used for tests
    testlist: VecDeque<StreamItem>,
}
}

impl Streamer {
    pub fn new(process_stream: tps::ProcessLineStream, refresh_delay: Duration) -> Result<Self> {
        Ok(Self {
            process: Some(process_stream),
            ticker: Some(IntervalStream::new(tokio::time::interval(
                refresh_delay.into(),
            ))),
            ..Self::default()
        })
    }

    #[cfg(test)]
    pub fn new_test(testlist: Vec<StreamItem>) -> Result<Self> {
        Ok(Self {
            testlist: VecDeque::from(testlist),
            ..Self::default()
        })
    }
}

impl Stream for Streamer {
    type Item = StreamItem;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        if let Some(item) = this.testlist.pop_front() {
            return Poll::Ready(Some(item));
        }
        if this.process.is_none() {
            // Keep returning None after the process is done
            return Poll::Ready(None);
        }
        if let Some(process) = this.process {
            match Pin::new(process).poll_next(cx) {
                Poll::Ready(Some(item)) => {
                    return Poll::Ready(Some(item.into()));
                }
                Poll::Ready(None) => {
                    *this.process = None;
                    *this.ticker = None;
                }
                Poll::Pending => {}
            }
        }
        let Some(ticker) = this.ticker else {
            unreachable!()
        };
        match Pin::new(ticker).poll_next(cx) {
            Poll::Ready(Some(item)) => Poll::Ready(Some(item.into())),
            _ => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use color_eyre::eyre::eyre;
    use color_eyre::Result;
    use tokio_stream::StreamExt;

    use crate::cli::Cli;
    use crate::sys_input::SysInputApi;
    use crate::sys_input::SysInputReal;

    use super::*;

    async fn stream_cmd(
        cmd: &[&str],
    ) -> Result<impl StreamExt<Item = StreamItem> + std::marker::Unpin + Send + 'static> {
        let duration = Duration::milliseconds(5000);
        let cli = Cli::try_parse_from(cmd)?;
        let sys_input = SysInputReal::default();
        let process_stream = sys_input.run_command(cli.get_command())?;
        Streamer::new(process_stream, duration)
    }

    async fn stream_next<T>(stream: &mut T) -> Result<StreamItem>
    where
        T: StreamExt<Item = StreamItem> + std::marker::Unpin + Send + 'static,
    {
        while let Some(item) = stream.next().await {
            match item {
                StreamItem::Tick => {
                    continue;
                }
                _ => {
                    return Ok(item);
                }
            }
        }
        Err(eyre!("no item received"))
    }

    #[tokio::test]
    async fn test_true() -> Result<()> {
        let mut stream = stream_cmd(&["ogle", "true"]).await?;
        let item = stream_next(&mut stream).await?;
        let StreamItem::Done(sts) = item else {
            return Err(eyre!("unexpected stream item {:?}", item));
        };
        assert!(sts.success());
        Ok(())
    }

    #[tokio::test]
    async fn test_false() -> Result<()> {
        let mut stream = stream_cmd(&["ogle", "false"]).await?;
        let item = stream_next(&mut stream).await?;
        let StreamItem::Done(sts) = item else {
            return Err(eyre!("unexpected stream item {:?}", item));
        };
        assert!(!sts.success());
        Ok(())
    }

    #[tokio::test]
    async fn test_echo() -> Result<()> {
        let mut stream = stream_cmd(&["ogle", "echo", "test"]).await?;
        let item = stream_next(&mut stream).await?;
        let StreamItem::LineOut(s) = item else {
            return Err(eyre!("unexpected stream item {:?}", item));
        };
        assert_eq!(s, "test");
        let item = stream_next(&mut stream).await?;
        let StreamItem::Done(sts) = item else {
            return Err(eyre!("unexpected stream item {:?}", item));
        };
        assert!(sts.success());
        Ok(())
    }

    #[tokio::test]
    async fn test_stderr() -> Result<()> {
        let mut stream = stream_cmd(&["ogle", "--", "/bin/sh", "-c", "echo test >&2"]).await?;
        let item = stream_next(&mut stream).await?;
        let StreamItem::LineErr(s) = item else {
            return Err(eyre!("unexpected stream item {:?}", item));
        };
        assert_eq!(s, "test");
        let item = stream_next(&mut stream).await?;
        let StreamItem::Done(sts) = item else {
            return Err(eyre!("unexpected stream item {:?}", item));
        };
        assert!(sts.success());
        Ok(())
    }

    #[tokio::test]
    async fn test_testlist() -> Result<()> {
        let list = vec![
            StreamItem::LineOut("stdout".into()),
            StreamItem::Tick,
            StreamItem::LineErr("stderr".into()),
            StreamItem::Tick,
            StreamItem::Done(ExitStatus::default()),
        ];
        let streamer = Streamer::new_test(list.clone())?;
        let streamed = streamer.collect::<Vec<_>>().await;
        assert_eq!(list, streamed);
        Ok(())
    }
}
