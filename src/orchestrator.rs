// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use color_eyre::eyre::eyre;
use color_eyre::Result;
use std::process::ExitStatus;
use tokio::time;
use tokio_stream::StreamExt;
use tracing::event;
use tracing::instrument;
use tracing::Level;

use crate::cli::Cli;
use crate::output_trait::Output;
use crate::stream::stream_create;
use crate::stream::StreamItem;

const REFRESH_DELAY: time::Duration = time::Duration::from_millis(250);

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
    let cli_period = time::Duration::from_secs(cli.period);
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
        let end = time::Instant::now() + cli_period;
        while time::Instant::now() < end {
            output.tick()?;
            time::sleep(REFRESH_DELAY).await;
        }
    }
}
