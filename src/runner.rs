// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use std::error::Error;
use std::io;
use std::io::BufRead;
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time;

use crate::cli::Cli;

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

pub fn run(cli: &Cli) -> Result<(), Box<dyn Error>> {
    let mut lastout = vec![];
    let period = time::Duration::from_secs(cli.period);
    let mut first = true;
    let mut cmd = buildcmd(&cli);
    loop {
        let mut child = cmd.spawn()?;
        let stdout = child.stdout.take().expect("error taking stdout");
        // let stderr = child.stderr().take().expect("error taking stdout");
        let bufstdout = io::BufReader::new(stdout);

        let mut currout = vec![];
        let mut different = false;
        for (iline, lineres) in bufstdout.lines().enumerate() {
            let line = lineres?;
            currout.push(line);
            if different {
                println!("{}", currout[iline]);
                continue;
            }
            if lastout.len() < iline + 1 || currout[iline] != lastout[iline] {
                // Print everything so far
                if !first {
                    println!();
                }
                println!("{}", chrono::offset::Local::now());
                for l in &currout {
                    println!("{}", l);
                }
                different = true;
                first = false;
            }
        }
        lastout = currout;
        if !different {
            thread::sleep(period);
        }
    }
}
