// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Module that wraps system functions used as inputs
//!
//! Wrapping this makes it very easy to test the whole program.

use color_eyre::Result;
use tokio::process::Command;
use tokio_process_stream as tps;

pub trait SysInputApi: std::fmt::Debug {
    fn run_command(&self, command: Command) -> Result<tps::ProcessLineStream>;
}

/// [`SysInputApi`] implementation of the real environment
#[derive(Debug, Default)]
pub struct SysInputReal {}

impl SysInputApi for SysInputReal {
    fn run_command(&self, command: Command) -> Result<tps::ProcessLineStream> {
        Ok(tps::ProcessStream::try_from(command)?)
    }
}
