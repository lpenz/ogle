// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use color_eyre::eyre::eyre;
use color_eyre::Result;
use tokio::time;
use tokio_stream::StreamExt;

use crate::cli::Cli;
use crate::output_simple::OutputSimple;
use crate::output_trait::Output;
use crate::stream::stream_create;
use crate::stream::StreamItem;

const REFRESH_DELAY: time::Duration = time::Duration::from_millis(250);

pub async fn stream_task<T>(output: &mut OutputSimple, mut stream: T) -> Result<()>
where
    T: StreamExt<Item = StreamItem> + std::marker::Unpin + Send + 'static,
{
    while let Some(item) = stream.next().await {
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
                return Ok(());
            }
            StreamItem::Err(e) => return Err(eyre!(e)),
        };
    }
    panic!("stream ended before process");
}

pub async fn run(cli: &Cli) -> Result<()> {
    let mut output = OutputSimple::new();
    let cli_period = time::Duration::from_secs(cli.period);
    loop {
        output.run_start()?;
        let stream = stream_create(cli, REFRESH_DELAY)?;
        let task = stream_task(&mut output, stream);
        task.await?;
        let end = time::Instant::now() + cli_period;
        while time::Instant::now() < end {
            output.tick()?;
            time::sleep(REFRESH_DELAY).await;
        }
    }
}
