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
mod sys_real;
use sys_real::Sys;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    color_eyre::install()?;
    let args = cli::Cli::parse();
    let sys = Sys::default();
    let output = output_sequence::OutputSequence::new(&sys, &args);
    orchestrator::run(&sys, &args, output).await?;
    Ok(())
}
