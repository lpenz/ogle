// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use clap::Parser;

use std::error::Error;

#[macro_use]
mod misc;

mod view;
mod view_sequence;

mod cli;
mod input_stream;
mod orchestrator;
mod progbar;
mod time_wrapper;

mod term_wrapper;

mod sys_input;
use sys_input::SysInputReal;

mod sys;
use sys::Sys;
mod sys_real;
use sys_real::SysReal;

mod output_sink;
#[cfg(test)]
mod sys_virtual;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    color_eyre::install()?;
    let args = cli::Cli::parse();
    let sys_input = SysInputReal::default();
    let mut sys = Sys::from(SysReal::default());
    let view = view_sequence::ViewSequence::new(&sys, &args);
    orchestrator::run(sys_input, &mut sys, &args, view.into()).await?;
    Ok(())
}
