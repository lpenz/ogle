// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use std::io;
use std::io::BufRead;
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time;

use crate::cli::Cli;

pub fn buildcmd(cli: &Cli) -> Command {
    let mut cmd = Command::new(&cli.command[0]);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    // cmd.stderr(Stdio::piped());
    cmd.args(cli.command.iter().skip(1));
    cmd
}

pub fn run(cli: &Cli) {
    let mut lastout = vec![];
    loop {
        let mut cmd = buildcmd(&cli);
        let mut child = cmd.spawn().expect("error running command");
        let stdout = child.stdout.take().expect("error taking stdout");
        // let stderr = child.stderr().take().expect("error taking stdout");
        let bufstdout = io::BufReader::new(stdout);

        let mut currout = vec![];
        let mut different = false;
        for (iline, lineres) in bufstdout.lines().enumerate() {
            let line = lineres.expect("error reading line");
            currout.push(line);
            if different {
                println!("{}", currout[iline]);
                continue;
            }
            if lastout.len() < iline + 1 || currout[iline] != lastout[iline] {
                // Print everything so far
                println!("{}", chrono::offset::Local::now());
                for l in &currout {
                    println!("{}", l);
                }
                different = true;
            }
        }
        lastout = currout;
        if !different {
            thread::sleep(time::Duration::from_secs(1));
        }
    }
}
