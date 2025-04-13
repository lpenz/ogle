// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use clap::Parser;

use std::error::Error;

#[macro_use]
mod misc;

mod cli;
mod input_stream;
mod orchestrator;
// mod progbar;
mod time_wrapper;

mod term_wrapper;

mod sys_input;
use sys_input::SysInputReal;

mod output_sink;

mod pipe;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    color_eyre::install()?;
    let args = cli::Cli::parse();
    let sys = SysInputReal::default();
    orchestrator::run(args, sys).await?;
    Ok(())
}
