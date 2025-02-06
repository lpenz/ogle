// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use color_eyre::Result;
use tracing::instrument;

use crate::cli::Cli;
use crate::input::InputStream;
use crate::output::output;
use crate::sys::SysApi;
use crate::time_wrapper::Duration;
use crate::view::Pipe;

#[instrument(level = "debug")]
pub async fn run<SI: SysApi>(cli: Cli, sys: SI) -> Result<()> {
    let refresh = Duration::milliseconds(250);
    let sleep = Duration::seconds(cli.period.into());
    let input = InputStream::new(sys.clone(), cli.get_cmd(), refresh, sleep)?;
    let view = Pipe::new(cli.get_cmd(), refresh, sleep, input);
    output(view).await
}
