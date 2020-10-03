// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use std::error::Error;
use std::io;
use std::io::BufRead;
use std::process;
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time;

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
    // cmd.stderr(Stdio::piped());
    cmd
}

pub fn first_run(cli: &Cli) -> Result<(process::ExitStatus, Vec<String>), Box<dyn Error>> {
    let mut cmd = buildcmd(&cli);
    println!(
        "$ {} # at {}",
        buildcmdline(&cli),
        chrono::offset::Local::now()
    );
    let mut child = cmd.spawn()?;
    let stdout = child.stdout.take().ok_or("error taking stdout")?;
    let bufstdout = io::BufReader::new(stdout);
    let mut lines = vec![];
    for line_result in bufstdout.lines() {
        let line = line_result?;
        println!("{}", line);
        lines.push(line);
    }
    let status = child.wait()?;
    Ok((status, lines))
}

pub fn run(cli: &Cli) -> Result<(), Box<dyn Error>> {
    let first_result = first_run(cli)?;
    let period = time::Duration::from_secs(cli.period);
    let mut last_lines = first_result.1;
    let mut cmd = buildcmd(&cli);
    loop {
        let mut child = cmd.spawn()?;
        let stdout = child.stdout.take().ok_or("error taking stdout")?;
        let bufstdout = io::BufReader::new(stdout);

        let mut lines = vec![];
        let mut different = false;
        for (iline, line_result) in bufstdout.lines().enumerate() {
            let line = line_result?;
            lines.push(line);
            if different {
                println!("{}", lines[iline]);
                continue;
            }
            if last_lines.len() < iline + 1 || lines[iline] != last_lines[iline] {
                // Print everything so far
                println!();
                println!(
                    "$ {} # at {}",
                    buildcmdline(&cli),
                    chrono::offset::Local::now()
                );
                for l in &lines {
                    println!("{}", l);
                }
                different = true;
            }
        }
        let status_res = child.wait();
        if cli.until_success {
            if let Ok(status) = status_res {
                if status.success() {
                    return Ok(());
                }
            }
        }
        last_lines = lines;
        if !different {
            thread::sleep(period);
        }
    }
}
