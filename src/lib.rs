// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use clap::Parser;

use std::error::Error;

mod output_trait;

mod output_simple;

mod cli;
mod misc;
mod orchestrator;
mod progbar;
mod runner;
mod stream;
mod timewrap;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install()?;
    let args = cli::Cli::parse();
    runner::run_loop(&args).await?;
    Ok(())
}
