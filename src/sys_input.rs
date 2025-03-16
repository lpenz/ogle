// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Module that wraps system functions used as inputs
//!
//! Wrapping this makes it very easy to test the whole program.

use color_eyre::Result;
use std::process::Stdio;
use tokio::process::Command;
use tokio_process_stream as tps;

/// A [`tokio::process::Command`] pseudo-wrapper that `impl Clone`.
#[derive(Clone)]
pub struct Cmd(Vec<String>);

impl Cmd {
    pub fn get_command(self) -> Command {
        let mut command = Command::new(&self.0[0]);
        command.args(self.0.iter().skip(1));
        command.stdin(Stdio::null());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        command
    }
}

impl From<Vec<String>> for Cmd {
    fn from(s: Vec<String>) -> Cmd {
        Self(s)
    }
}

pub trait SysInputApi: std::fmt::Debug {
    fn run_command(&self, command: Cmd) -> Result<tps::ProcessLineStream>;
}

/// [`SysInputApi`] implementation of the real environment
#[derive(Debug, Default)]
pub struct SysInputReal {}

impl SysInputApi for SysInputReal {
    fn run_command(&self, cmd: Cmd) -> Result<tps::ProcessLineStream> {
        Ok(tps::ProcessStream::try_from(cmd.get_command())?)
    }
}
