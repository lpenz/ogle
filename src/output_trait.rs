// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use enum_dispatch::enum_dispatch;

use color_eyre::Result;
use mockall::automock;
use std::process::ExitStatus;

use crate::output_sequence::OutputSequence;
use crate::output_simple::OutputSimple;

use crate::sys_api::SysApi;

#[enum_dispatch]
#[derive(Debug)]
pub enum OutputEnum {
    OutputSimple,
    OutputSequence,
    MockOutput,
}

#[enum_dispatch(OutputEnum)]
#[automock]
pub trait Output {
    fn run_start<Sys: SysApi + 'static>(&mut self, sys: &mut Sys) -> Result<()>;
    fn run_end<Sys: SysApi + 'static>(
        &mut self,
        sys: &mut Sys,
        exitstatus: ExitStatus,
    ) -> Result<()>;
    fn out_line<Sys: SysApi + 'static>(&mut self, sys: &mut Sys, line: String) -> Result<()>;
    fn err_line<Sys: SysApi + 'static>(&mut self, sys: &mut Sys, line: String) -> Result<()>;
    fn tick<Sys: SysApi + 'static>(&mut self, sys: &mut Sys) -> Result<()>;
}
