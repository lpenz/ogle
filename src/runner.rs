// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use std::io::stdout;
use std::io::Write;
use std::process::Command;
use std::process::Stdio;

use crate::cli::Cli;

pub fn buildcmd(cli: &Cli) -> Command {
    let mut cmd = Command::new(&cli.command[0]);
    cmd.stdin(Stdio::null());
    cmd.args(cli.command.iter().skip(1));
    cmd
}

pub fn run(cli: &Cli) {
    let mut lastout = None;
    loop {
        let mut cmd = Command::new(&cli.command[0]);
        let out = cmd.output().ok();
        if out != lastout {
            lastout = out;
            if let Some(o) = &lastout {
                println!("{}", chrono::offset::Local::now());
                stdout()
                    .write_all(&o.stdout)
                    .expect("error writing to stdout");
            }
        }
    }
}
