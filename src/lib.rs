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

mod differ;

mod pipe;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::ACTIVE)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let args = cli::Cli::parse();
    let sys = SysInputReal::default();
    orchestrator::run(args, sys).await?;
    Ok(())
}
