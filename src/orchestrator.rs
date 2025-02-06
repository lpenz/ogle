// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use color_eyre::Result;
use color_eyre::eyre::eyre;
use std::process::ExitStatus;
use tokio_stream::StreamExt;
use tracing::Level;
use tracing::event;
use tracing::instrument;

use crate::cli::Cli;
use crate::output_trait::Output;
use crate::stream::StreamItem;
use crate::stream::stream_create;
use crate::sys_api::SysApi;
use crate::time_wrapper::Duration;

const REFRESH_DELAY: Duration = Duration::milliseconds(250);

#[instrument(level = "debug", skip_all)]
pub async fn stream_task<Sys: SysApi + 'static, O, T>(
    sys: &Sys,
    output: &mut O,
    mut stream: T,
) -> Result<Option<ExitStatus>>
where
    O: Output,
    T: StreamExt<Item = StreamItem> + std::marker::Unpin,
{
    while let Some(item) = stream.next().await {
        event!(Level::DEBUG, item = ?item, "received");
        match item {
            StreamItem::LineOut(line) => {
                output.out_line(sys, line)?;
            }
            StreamItem::LineErr(line) => {
                output.err_line(sys, line)?;
            }
            StreamItem::Tick => {
                output.tick(sys)?;
            }
            StreamItem::Done(sts) => {
                output.run_end(sys, &sts)?;
                return Ok(Some(sts));
            }
            StreamItem::Err(e) => return Err(eyre!(e)),
        };
    }
    panic!("stream ended before process");
}

#[instrument(level = "debug")]
pub async fn run<Sys: SysApi + 'static, O: Output + std::fmt::Debug>(
    sys: &Sys,
    cli: &Cli,
    mut output: O,
) -> Result<()> {
    let cli_period = Duration::seconds(cli.period.into());
    loop {
        output.run_start(sys)?;
        let stream = stream_create(cli, REFRESH_DELAY)?;
        let task = stream_task(sys, &mut output, stream);
        if let Some(result) = task.await? {
            if (cli.until_success && result.success()) || (cli.until_failure && !result.success()) {
                return Ok(());
            }
        }
        // Sleep
        let end = &sys.now() + &cli_period;
        while sys.now() < end {
            output.tick(sys)?;
            tokio::time::sleep(REFRESH_DELAY.into()).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::output_trait::MockOutput;
    use crate::sys_api::MockSysApi;
    use clap::Parser;

    #[tokio::test]
    async fn test_true() -> Result<()> {
        let mut omock = MockOutput::new();
        omock
            .expect_run_start()
            .times(1)
            .returning(|_: &MockSysApi| Ok(()));
        omock.expect_tick().returning(|_: &MockSysApi| Ok(()));
        omock
            .expect_run_end()
            .times(1)
            .returning(|_: &MockSysApi, s| {
                assert!(s.success());
                Ok(())
            });
        let cli = Cli::try_parse_from(["ogle", "-z", "--", "true"])?;
        let sys = MockSysApi::default();
        run(&sys, &cli, omock).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_false() -> Result<()> {
        let mut omock = MockOutput::new();
        omock
            .expect_run_start()
            .times(1)
            .returning(|_: &MockSysApi| Ok(()));
        omock.expect_tick().returning(|_: &MockSysApi| Ok(()));
        omock
            .expect_run_end()
            .times(1)
            .returning(|_: &MockSysApi, s| {
                assert!(!s.success());
                Ok(())
            });
        let cli = Cli::try_parse_from(["ogle", "-e", "--", "false"])?;
        let sys = MockSysApi::default();
        run(&sys, &cli, omock).await?;
        Ok(())
    }
}
