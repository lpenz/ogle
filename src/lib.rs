// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use clap::Parser;
use std::error::Error;

mod cli;
mod misc;
mod progbar;
mod runner;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::Cli::parse();
    runner::run_loop(&args).await?;
    Ok(())
}
