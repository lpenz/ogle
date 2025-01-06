// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use color_eyre::Result;
use std::convert::TryFrom;
use std::process::ExitStatus;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time;
use tokio_process_stream as tps;
use tokio_stream::wrappers::IntervalStream;
use tokio_stream::StreamExt;

use crate::cli::Cli;

#[derive(Debug)]
pub enum StreamItem {
    Line(String),
    Done(ExitStatus),
    Tick,
}

impl From<tps::Item<String>> for StreamItem {
    fn from(item: tps::Item<String>) -> Self {
        match item {
            tps::Item::Stdout(l) => StreamItem::Line(l),
            tps::Item::Stderr(l) => StreamItem::Line(l),
            tps::Item::Done(s) => StreamItem::Done(s.unwrap()),
            // TODO: add error to get rid of this unwrap ^
        }
    }
}

impl From<time::Instant> for StreamItem {
    fn from(_: time::Instant) -> Self {
        StreamItem::Tick
    }
}

fn buildcmd(cli: &Cli) -> Command {
    let mut cmd = Command::new(&cli.command[0]);
    cmd.args(cli.command.iter().skip(1));
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd
}

pub fn stream_create(
    cli: &Cli,
    refresh_delay: time::Duration,
) -> Result<impl StreamExt<Item = StreamItem> + std::marker::Unpin + Send + 'static> {
    let cmd = buildcmd(cli);
    let procstream = tps::ProcessStream::try_from(cmd)?.map(StreamItem::from);
    let ticker = IntervalStream::new(time::interval(refresh_delay));
    Ok(procstream.merge(ticker.map(StreamItem::from)))
}
