// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use enum_dispatch::enum_dispatch;

use color_eyre::Result;
use mockall::automock;

use crate::sys_real::SysReal;
use crate::time_wrapper::Instant;

#[enum_dispatch]
#[derive(Debug)]
pub enum Sys {
    SysReal,
    MockSysApi,
}

#[enum_dispatch(Sys)]
#[automock]
pub trait SysApi: std::fmt::Debug {
    fn now(&self) -> Instant;
    fn width(&self) -> usize;
    fn log_line(&mut self, line: &str) -> Result<()>;
    fn update_status(&mut self, status: &str) -> Result<()>;
}
