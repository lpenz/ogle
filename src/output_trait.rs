// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use color_eyre::Result;
use std::process::ExitStatus;

pub trait Output {
    fn run_start(&mut self) -> Result<()>;
    fn run_end(&mut self, exitstatus: &ExitStatus) -> Result<()>;
    fn out_line(&mut self, line: String) -> Result<()>;
    fn err_line(&mut self, line: String) -> Result<()>;
    fn tick(&mut self) -> Result<()>;
}
