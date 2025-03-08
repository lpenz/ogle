// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use clap::Parser;

use std::error::Error;

#[macro_use]
mod misc;

mod output_trait;

mod output_sequence;
mod output_simple;

mod cli;
mod orchestrator;
mod progbar;
mod stream;
mod time_wrapper;

mod sys_api;
use sys_api::Sys;
mod sys_real;
use sys_real::SysReal;

#[cfg(test)]
mod sys_virtual;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    color_eyre::install()?;
    let args = cli::Cli::parse();
    let mut sys = Sys::from(SysReal::default());
    let output = output_sequence::OutputSequence::new(&sys, &args);
    orchestrator::run(&mut sys, &args, output.into()).await?;
    Ok(())
}
