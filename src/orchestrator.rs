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
use crate::output_trait::OutputEnum;
use crate::stream::stream_create;
use crate::stream::StreamItem;
use crate::sys_api::Sys;
use crate::sys_api::SysApi;
use crate::time_wrapper::Duration;

const REFRESH_DELAY: Duration = Duration::milliseconds(250);

#[instrument(level = "debug", skip_all)]
pub async fn stream_task<T>(
    sys: &mut Sys,
    output: &mut OutputEnum,
    mut stream: T,
) -> Result<Option<ExitStatus>>
where
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
                output.run_end(sys, sts)?;
                return Ok(Some(sts));
            }
            StreamItem::Err(e) => return Err(eyre!(e)),
        };
    }
    panic!("stream ended before process");
}

#[instrument(level = "debug")]
pub async fn run(sys: &mut Sys, cli: &Cli, mut output: OutputEnum) -> Result<()> {
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

    use crate::output_sequence::OutputSequence;
    use crate::sys_virtual::SysVirtual;
    use clap::Parser;

    #[tokio::test]
    async fn test_true() -> Result<()> {
        let o = OutputSequence::default();
        let cli = Cli::try_parse_from(["ogle", "-z", "--", "true"])?;
        let mut sys = SysVirtual::default().into();
        run(&mut sys, &cli, OutputEnum::from(o)).await?;
        let Sys::SysVirtual(sys) = sys else {
            unreachable!()
        };
        assert_eq!(
            sys.log,
            vec!["<O> 1970-01-01 00:00:00 first execution", "+ "]
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_false() -> Result<()> {
        let o = OutputSequence::default();
        let cli = Cli::try_parse_from(["ogle", "-e", "--", "false"])?;
        let mut sys = SysVirtual::default().into();
        run(&mut sys, &cli, OutputEnum::from(o)).await?;
        let Sys::SysVirtual(sys) = sys else {
            unreachable!()
        };
        assert_eq!(
            sys.log,
            vec!["<O> 1970-01-01 00:00:00 first execution", "+ "]
        );
        Ok(())
    }
}
