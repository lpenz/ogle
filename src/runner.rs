// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use anyhow::Result;
use std::process::ExitStatus;
use std::process::Stdio;
use tokio::io;
use tokio::io::AsyncBufReadExt;
use tokio::process::Command;
use tokio::stream::StreamExt;
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

#[derive(Debug, Default)]
pub struct RunData {
    status: Option<ExitStatus>,
    output: Vec<String>,
}

impl RunData {
    pub fn success(&self) -> bool {
        self.status.map_or(false, |s| s.success())
    }
}

pub async fn stream_task<T>(
    cmdline: String,
    last_lines: Vec<String>,
    mut stream: T,
) -> Result<Vec<String>>
where
    T: StreamExt<Item = Result<String>> + std::marker::Unpin + Send + 'static,
{
    let mut lines = vec![];
    let mut different = false;
    let mut iline = 0;
    while let Some(line0) = stream.next().await {
        let line = line0.map_err(anyhow::Error::from)?;
        lines.push(line);
        if different {
            println!("{}", lines[iline]);
            iline += 1;
            continue;
        }
        if last_lines.len() < iline + 1 || lines[iline] != last_lines[iline] {
            // Print everything so far
            println!();
            println!("$ {} # at {}", cmdline, chrono::offset::Local::now());
            for l in &lines {
                println!("{}", l);
            }
            different = true;
        }
        iline += 1;
    }
    Ok(lines)
}

pub fn std_to_stream<T: tokio::io::AsyncRead>(
    name: &str,
    stdopt: Option<T>,
) -> Result<impl StreamExt<Item = Result<String>>> {
    let std = stdopt.ok_or_else(|| anyhow::anyhow!("error taking {:?}", name))?;
    let br = io::BufReader::new(std);
    Ok(br.lines().map(|r| r.map_err(anyhow::Error::from)))
}

pub async fn run_once(cli: &Cli, last_rundata: RunData) -> Result<RunData> {
    let mut cmd = buildcmd(&cli);
    let mut child = cmd.spawn()?;
    let stdout_stream = std_to_stream("stdout", child.stdout.take())?;
    let stderr_stream = std_to_stream("stderr", child.stderr.take())?;
    let both_stream = stdout_stream.merge(stderr_stream);
    let both_task = stream_task(buildcmdline(cli), last_rundata.output, both_stream);
    tokio::spawn(async move {
        let (status, vecboth) = tokio::join!(child, both_task);
        Ok(RunData {
            status: Some(status?),
            output: vecboth?,
        })
    })
    .await?
}

pub async fn run_loop(cli: &Cli) -> Result<()> {
    let mut last_rundata = run_once(cli, RunData::default()).await?;
    if cli.until_success && last_rundata.success() {
        return Ok(());
    }
    let period = time::Duration::from_secs(cli.period);
    loop {
        let rundata = run_once(cli, last_rundata).await?;
        if cli.until_success && rundata.success() {
            return Ok(());
        }
        last_rundata = rundata;
        time::delay_for(period).await;
    }
}
