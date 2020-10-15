// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use anyhow::Result;
use indicatif;
use std::process::ExitStatus;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tokio::io;
use tokio::io::AsyncBufReadExt;
use tokio::process::Command;
use tokio::stream::StreamExt;
use tokio::sync::mpsc;
use tokio::time;

use crate::cli::Cli;

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

pub fn progbar(duration: time::Duration, position: time::Duration) -> indicatif::ProgressBar {
    let pb = indicatif::ProgressBar::hidden();
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{msg} {bar:40.cyan/blue}")
            .progress_chars("##-"),
    );
    pb.set_length(duration.as_millis() as u64);
    pb.set_position(position.as_millis() as u64);
    pb.set_draw_target(indicatif::ProgressDrawTarget::stderr());
    pb
}

#[derive(Debug)]
pub enum StreamItem {
    Line(String),
    Err(anyhow::Error),
    Tick,
}

pub async fn stream_task<T>(
    cmdline: String,
    last_lines: Vec<String>,
    last_period: time::Duration,
    mut stream: T,
) -> Result<Vec<String>>
where
    T: StreamExt<Item = StreamItem> + std::marker::Unpin + Send + 'static,
{
    let mut lines = vec![];
    let mut different = false;
    let mut iline = 0;
    let mut pb = progbar(last_period, time::Duration::default());
    pb.set_message("running");
    let start = time::Instant::now();
    while let Some(item) = stream.next().await {
        match item {
            StreamItem::Line(line) => {
                lines.push(line);
                if different {
                    pb.finish_and_clear();
                    println!("{}", lines[iline]);
                    pb = progbar(last_period, time::Instant::now() - start);
                    pb.set_message("running");
                    //pb.set_position((time::Instant::now() - start).as_millis() as u64);
                    iline += 1;
                    continue;
                }
                if last_lines.len() < iline + 1 || lines[iline] != last_lines[iline] {
                    // Print everything so far
                    pb.finish_and_clear();
                    println!();
                    println!("$ {} # at {}", cmdline, chrono::offset::Local::now());
                    for l in &lines {
                        println!("{}", l);
                    }
                    different = true;
                    pb = progbar(last_period, time::Instant::now() - start);
                    pb.set_message("running");
                    // pb.set_position((time::Instant::now() - start).as_millis() as u64);
                }
                iline += 1;
            }
            StreamItem::Tick => {
                pb.set_position((time::Instant::now() - start).as_millis() as u64);
            }
            _ => { /* ignore read errors */ }
        }
    }
    Ok(lines)
}

pub fn std_to_stream<T: tokio::io::AsyncRead>(
    name: &str,
    stdopt: Option<T>,
) -> Result<impl StreamExt<Item = StreamItem>> {
    let std = stdopt.ok_or_else(|| anyhow::anyhow!("error taking {:?}", name))?;
    let br = io::BufReader::new(std);
    Ok(br.lines().map(|r| match r {
        Ok(l) => StreamItem::Line(l),
        Err(e) => StreamItem::Err(anyhow::Error::from(e)),
    }))
}

pub async fn ticker(
    mut tx: tokio::sync::mpsc::Sender<StreamItem>,
    done_guard: &Arc<Mutex<bool>>,
) -> Result<()> {
    let delay = time::Duration::from_millis(100);
    loop {
        time::delay_for(delay).await;
        tx.send(StreamItem::Tick).await?;
        let done = done_guard.lock().unwrap();
        if *done {
            break;
        }
    }
    Ok(())
}

pub async fn wait(
    child: tokio::process::Child,
    done_guard: &Arc<Mutex<bool>>,
) -> Result<ExitStatus> {
    let status = child.await;
    let mut done = done_guard.lock().unwrap();
    *done = true;
    status.map_err(|e| anyhow::anyhow!(e))
}

pub async fn run_once(cli: &Cli, last_rundata: RunData) -> Result<RunData> {
    let mut cmd = buildcmd(&cli);
    let mut child = cmd.spawn()?;
    let start = time::Instant::now();
    let stdout_stream = std_to_stream("stdout", child.stdout.take())?;
    let stderr_stream = std_to_stream("stderr", child.stderr.take())?;
    let (tx, rx) = mpsc::channel(2);
    let stream = stdout_stream.merge(stderr_stream).merge(rx);
    let task = stream_task(
        buildcmdline(cli),
        last_rundata.output,
        last_rundata.duration,
        stream,
    );
    let done_guard = Arc::new(Mutex::new(false));
    let ticker = ticker(tx, &done_guard);
    let wait = wait(child, &done_guard);
    let (status, vecboth, _) = tokio::join!(wait, task, ticker);
    Ok(RunData {
        status: Some(status?),
        output: vecboth?,
        duration: time::Instant::now() - start,
    })
}

pub async fn run_loop(cli: &Cli) -> Result<()> {
    let mut last_rundata = run_once(cli, RunData::default()).await?;
    // let duration = time::Instant::now() - start;
    if cli.until_success && last_rundata.success() {
        return Ok(());
    }
    let delay = time::Duration::from_millis(100);
    loop {
        let rundata = run_once(cli, last_rundata).await?;
        if cli.until_success && rundata.success() {
            return Ok(());
        }
        last_rundata = rundata;
        let cli_period = time::Duration::from_secs(cli.period);
        let progbar = progbar(cli_period, time::Duration::default());
        let mut now = time::Instant::now();
        let end = now + cli_period;
        while now < end {
            time::delay_for(delay).await;
            let nownew = time::Instant::now();
            progbar.set_message("sleeping");
            progbar.inc((nownew - now).as_millis() as u64);
            now = nownew;
        }
    }
}
