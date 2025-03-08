// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use enum_dispatch::enum_dispatch;

use color_eyre::Result;
use mockall::automock;
use std::process::ExitStatus;

use crate::output_sequence::OutputSequence;

use crate::sys_api::Sys;

#[enum_dispatch]
#[derive(Debug)]
pub enum OutputEnum {
    OutputSequence,
    MockOutput,
}

#[enum_dispatch(OutputEnum)]
#[automock]
pub trait Output {
    fn run_start(&mut self, sys: &mut Sys) -> Result<()>;
    fn run_end(&mut self, sys: &mut Sys, exitstatus: ExitStatus) -> Result<()>;
    fn out_line(&mut self, sys: &mut Sys, line: String) -> Result<()>;
    fn err_line(&mut self, sys: &mut Sys, line: String) -> Result<()>;
    fn tick(&mut self, sys: &mut Sys) -> Result<()>;
}
