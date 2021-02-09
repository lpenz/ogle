// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use anyhow::Result;
use std::process::ExitStatus;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time;
use tokio_stream::StreamExt;

use crate::childstream;
use crate::cli::Cli;
use crate::progbar::Progbar;
use crate::ticker::Ticker;

const REFRESH_DELAY: time::Duration = time::Duration::from_millis(250);

pub fn buildcmdline(cli: &Cli) -> String {
    if cli.shell {
        format!("/bin/sh -c \"{}\"", cli.command[0].as_str())
    } else {
        cli.command.join(" ")
    }
}

pub fn buildcmd(cli: &Cli) -> Command {
    let mut cmd = if cli.shell {
        let mut cmd = Command::new("/bin/sh");
        cmd.args(&["-c"]);
        cmd.args(&[cli.command[0].as_str()]);
        cmd
    } else {
        let mut cmd = Command::new(&cli.command[0]);
        cmd.args(cli.command.iter().skip(1));
        cmd
    };
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd
}

#[derive(Debug, Default, Clone)]
pub struct RunData {
    status: Option<ExitStatus>,
    output: Vec<String>,
    duration: time::Duration,
}

impl RunData {
    pub fn success(&self) -> bool {
        self.status.map_or(false, |s| s.success())
    }
}

#[derive(Debug)]
pub enum StreamItem {
    Line(String),
    Done(ExitStatus),
    Tick,
}

pub fn print_backlog(pb: &mut Progbar, cmdline: &str, lines: &[String]) -> Result<()> {
    pb.hide()?;
    println!();
    println!("=> changed at {}", chrono::offset::Local::now());
    println!("+ {}", cmdline);
    for l in lines {
        println!("{}", l);
    }
    pb.show()?;
    Ok(())
}

pub async fn stream_task<T>(
    cmdline: &str,
    last_lines: Vec<String>,
    last_period: time::Duration,
    mut stream: T,
    pb: &mut Progbar,
    ticker: &Ticker,
) -> Result<(ExitStatus, Vec<String>)>
where
    T: StreamExt<Item = StreamItem> + std::marker::Unpin + Send + 'static,
{
    let mut lines = vec![];
    let mut different = false;
    let mut nlines = 0;
    pb.set_running(last_period);
    while let Some(item) = stream.next().await {
        let (stsopt, lineopt) = match item {
            StreamItem::Line(line) => (None, Some(line)),
            StreamItem::Tick => {
                pb.refresh()?;
                (None, None)
            }
            StreamItem::Done(sts) => (
                Some(sts),
                if let Some(code) = sts.code() {
                    if code == 0 {
                        None
                    } else {
                        Some(format!("=> exit code {}", code))
                    }
                } else {
                    Some("=> error getting exit code".to_string())
                },
            ),
        };
        if let Some(line) = lineopt {
            lines.push(line);
            nlines += 1;
            if different {
                pb.hide()?;
                println!("{}", lines[nlines - 1]);
                pb.show()?;
            } else if last_lines.len() < nlines || lines[nlines - 1] != last_lines[nlines - 1] {
                // Print everything so far
                print_backlog(pb, cmdline, &lines)?;
                different = true;
            }
        }
        if let Some(sts) = stsopt {
            /* Process is done, check if we got less lines: */
            if !different && last_lines.len() > nlines {
                print_backlog(pb, cmdline, &lines)?;
            }
            ticker.close();
            return Ok((sts, lines));
        }
    }
    panic!("stream ended before process");
}

pub async fn run_once(cli: &Cli, last_rundata: RunData, pb: &mut Progbar) -> Result<RunData> {
    let cmd = buildcmd(&cli);
    let start = time::Instant::now();
    let childstream = childstream::ChildStream::from_command(cmd)?.map(|i| match i {
        childstream::Item::Stdout(l) => StreamItem::Line(l),
        childstream::Item::Stderr(l) => StreamItem::Line(l),
        childstream::Item::Done(s) => StreamItem::Done(s),
    });
    let ticker = Ticker::default();
    let stream = childstream.merge(
        ticker
            .create_stream(REFRESH_DELAY)
            .map(|_| StreamItem::Tick),
    );
    let cmdline = buildcmdline(cli);
    let task = stream_task(
        &cmdline,
        last_rundata.output,
        last_rundata.duration,
        stream,
        pb,
        &ticker,
    );
    let (status, vecboth) = task.await?;
    Ok(RunData {
        status: Some(status),
        output: vecboth,
        duration: time::Instant::now() - start,
    })
}

pub async fn run_loop(cli: &Cli) -> Result<()> {
    let mut pb = Progbar::default();
    let mut last_rundata = run_once(cli, RunData::default(), &mut pb).await?;
    if cli.until_success && last_rundata.success() {
        return Ok(());
    }
    let cli_period = time::Duration::from_secs(cli.period);
    loop {
        let rundata = run_once(cli, last_rundata, &mut pb).await?;
        if cli.until_success && rundata.success() {
            return Ok(());
        }
        last_rundata = rundata;
        pb.set_sleep(cli_period);
        let end = time::Instant::now() + cli_period;
        while time::Instant::now() < end {
            pb.refresh()?;
            time::sleep(REFRESH_DELAY).await;
        }
    }
}
