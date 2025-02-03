// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use color_eyre::Result;
use std::convert::TryFrom;
use std::process::ExitStatus;
use tokio_process_stream as tps;
use tokio_stream::wrappers::IntervalStream;
use tokio_stream::StreamExt;

use crate::cli::Cli;
use crate::time_wrapper::Duration;

#[derive(Debug)]
pub enum StreamItem {
    LineOut(String),
    LineErr(String),
    Done(ExitStatus),
    Err(std::io::Error),
    Tick,
}

impl From<tps::Item<String>> for StreamItem {
    fn from(item: tps::Item<String>) -> Self {
        match item {
            tps::Item::Stdout(l) => StreamItem::LineOut(l),
            tps::Item::Stderr(l) => StreamItem::LineErr(l),
            tps::Item::Done(Ok(sts)) => StreamItem::Done(sts),
            tps::Item::Done(Err(e)) => StreamItem::Err(e),
        }
    }
}

impl From<tokio::time::Instant> for StreamItem {
    fn from(_: tokio::time::Instant) -> Self {
        StreamItem::Tick
    }
}

pub fn stream_create(
    cli: &Cli,
    refresh_delay: Duration,
) -> Result<impl StreamExt<Item = StreamItem> + std::marker::Unpin + Send + 'static> {
    let cmd = cli.get_command();
    let procstream = tps::ProcessStream::try_from(cmd)?.map(StreamItem::from);
    let ticker = IntervalStream::new(tokio::time::interval(refresh_delay.into()));
    Ok(procstream.merge(ticker.map(StreamItem::from)))
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use color_eyre::eyre::eyre;
    use color_eyre::Result;
    use tokio_stream::StreamExt;

    use crate::cli::Cli;

    use super::*;

    async fn stream_cmd(
        cmd: &[&str],
    ) -> Result<impl StreamExt<Item = StreamItem> + std::marker::Unpin + Send + 'static> {
        let duration = Duration::milliseconds(5000);
        stream_create(&Cli::try_parse_from(cmd)?, duration)
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
}
