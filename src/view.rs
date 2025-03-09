// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use enum_dispatch::enum_dispatch;

use color_eyre::Result;
use std::process::ExitStatus;

use crate::view_sequence::ViewSequence;

use crate::sys::Sys;

#[enum_dispatch]
#[derive(Debug)]
pub enum View {
    ViewSequence,
}

#[enum_dispatch(View)]
pub trait ViewApi {
    fn run_start(&mut self, sys: &mut Sys) -> Result<()>;
    fn run_end(&mut self, sys: &mut Sys, exitstatus: ExitStatus) -> Result<()>;
    fn out_line(&mut self, sys: &mut Sys, line: String) -> Result<()>;
    fn err_line(&mut self, sys: &mut Sys, line: String) -> Result<()>;
    fn tick(&mut self, sys: &mut Sys) -> Result<()>;
}
