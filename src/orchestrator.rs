// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use color_eyre::eyre::eyre;
use color_eyre::Result;
use std::process::ExitStatus;
use tokio_stream::StreamExt;
use tracing::event;
use tracing::instrument;
use tracing::Level;

use crate::cli::Cli;
use crate::output_trait::Output;
use crate::stream::stream_create;
use crate::stream::StreamItem;
use crate::time_wrapper::Duration;
use crate::time_wrapper::Instant;

const REFRESH_DELAY: Duration = Duration::milliseconds(250);

#[instrument(level = "debug", skip_all)]
pub async fn stream_task<O, T>(output: &mut O, mut stream: T) -> Result<Option<ExitStatus>>
where
    O: Output,
    T: StreamExt<Item = StreamItem> + std::marker::Unpin,
{
    while let Some(item) = stream.next().await {
        event!(Level::DEBUG, item = ?item, "received");
        match item {
            StreamItem::LineOut(line) => {
                output.out_line(line)?;
            }
            StreamItem::LineErr(line) => {
                output.err_line(line)?;
            }
            StreamItem::Tick => {
                output.tick()?;
            }
            StreamItem::Done(sts) => {
                output.run_end(&sts)?;
                return Ok(Some(sts));
            }
            StreamItem::Err(e) => return Err(eyre!(e)),
        };
    }
    panic!("stream ended before process");
}

#[instrument(level = "debug")]
pub async fn run<O: Output + std::fmt::Debug>(cli: &Cli, mut output: O) -> Result<()> {
    let cli_period = Duration::seconds(cli.period.into());
    loop {
        output.run_start()?;
        let stream = stream_create(cli, REFRESH_DELAY)?;
        let task = stream_task(&mut output, stream);
        if let Some(result) = task.await? {
            if (cli.until_success && result.success()) || (cli.until_failure && !result.success()) {
                return Ok(());
            }
        }
        // Sleep
        let end = &Instant::now() + &cli_period;
        while Instant::now() < end {
            output.tick()?;
            tokio::time::sleep(REFRESH_DELAY.into()).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::output_trait::MockOutput;
    use clap::Parser;

    #[tokio::test]
    async fn test_true() -> Result<()> {
        let mut mock = MockOutput::new();
        mock.expect_run_start().times(1).returning(|| Ok(()));
        mock.expect_tick().returning(|| Ok(()));
        mock.expect_run_end().times(1).returning(|s| {
            assert!(s.success());
            Ok(())
        });
        let cli = Cli::try_parse_from(["ogle", "-z", "--", "true"])?;
        run(&cli, mock).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_false() -> Result<()> {
        let mut mock = MockOutput::new();
        mock.expect_run_start().times(1).returning(|| Ok(()));
        mock.expect_tick().returning(|| Ok(()));
        mock.expect_run_end().times(1).returning(|s| {
            assert!(!s.success());
            Ok(())
        });
        let cli = Cli::try_parse_from(["ogle", "-e", "--", "false"])?;
        run(&cli, mock).await?;
        Ok(())
    }
}
