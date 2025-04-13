// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use color_eyre::Result;
use tracing::instrument;

use crate::cli::Cli;
use crate::input_stream::InputStream;
use crate::output_sink::output_sink;
use crate::pipe::Pipe;
use crate::sys_input::SysInputApi;
use crate::time_wrapper::Duration;

#[instrument(level = "debug")]
pub async fn run<SI: SysInputApi>(cli: Cli, sys: SI) -> Result<()> {
    let cli_period = Duration::seconds(cli.period.into());
    let input_stream = InputStream::new(sys.clone(), cli.get_cmd(), cli_period)?;
    let pipe = Pipe::from(input_stream);
    output_sink(pipe).await
}
