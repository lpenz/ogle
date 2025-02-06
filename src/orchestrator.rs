// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use color_eyre::Result;
use tracing::instrument;

use crate::cli::Cli;
use crate::input_stream::InputStream;
use crate::output::output;
use crate::pipe::Pipe;
use crate::sys::SysApi;
use crate::time_wrapper::Duration;

#[instrument(level = "debug")]
pub async fn run<SI: SysApi>(cli: Cli, sys: SI) -> Result<()> {
    let refresh = Duration::milliseconds(250);
    let sleep = Duration::seconds(cli.period.into());
    let input_stream = InputStream::new(sys.clone(), cli.get_cmd(), refresh, sleep)?;
    let pipe = Pipe::new(cli.get_cmd(), refresh, sleep, input_stream);
    output(pipe).await
}
