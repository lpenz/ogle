// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use anyhow::Result;
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
use crate::progbar::Progbar;

const REFRESH_DELAY: time::Duration = time::Duration::from_millis(150);

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
    Err(anyhow::Error),
    Tick,
}

pub fn print_backlog(pb: &mut Progbar, cmdline: &str, lines: &[String]) {
    pb.hide();
    println!();
    println!("$ {} # at {}", cmdline, chrono::offset::Local::now());
    for l in lines {
        println!("{}", l);
    }
    pb.show();
}

pub async fn stream_task<T>(
    cmdline: &str,
    last_lines: Vec<String>,
    last_period: time::Duration,
    mut stream: T,
    pb: &mut Progbar,
) -> Result<Vec<String>>
where
    T: StreamExt<Item = StreamItem> + std::marker::Unpin + Send + 'static,
{
    let mut lines = vec![];
    let mut different = false;
    let mut nlines = 0;
    pb.set_running(last_period);
    while let Some(item) = stream.next().await {
        match item {
            StreamItem::Line(line) => {
                lines.push(line);
                nlines += 1;
                if different {
                    pb.hide();
                    println!("{}", lines[nlines - 1]);
                    pb.show();
                } else if last_lines.len() < nlines || lines[nlines - 1] != last_lines[nlines - 1] {
                    // Print everything so far
                    print_backlog(pb, cmdline, &lines);
                    different = true;
                }
            }
            StreamItem::Tick => {
                pb.refresh();
            }
            _ => { /* ignore read errors */ }
        }
    }
    /* Process is done, check if we got less lines: */
    if !different && last_lines.len() > nlines {
        print_backlog(pb, cmdline, &lines);
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
    loop {
        time::delay_for(REFRESH_DELAY).await;
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

pub async fn run_once(cli: &Cli, last_rundata: RunData, pb: &mut Progbar) -> Result<RunData> {
    let mut cmd = buildcmd(&cli);
    let mut child = cmd.spawn()?;
    let start = time::Instant::now();
    let stdout_stream = std_to_stream("stdout", child.stdout.take())?;
    let stderr_stream = std_to_stream("stderr", child.stderr.take())?;
    let (tx, rx) = mpsc::channel(2);
    let stream = stdout_stream.merge(stderr_stream).merge(rx);
    let cmdline = buildcmdline(cli);
    let task = stream_task(
        &cmdline,
        last_rundata.output,
        last_rundata.duration,
        stream,
        pb,
    );
    // We use done_guard mutex to protect stdou/err
    #[allow(clippy::mutex_atomic)]
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
            pb.refresh();
            time::delay_for(REFRESH_DELAY).await;
        }
    }
}
