// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use color_eyre::Result;
use tracing::instrument;

use crate::cli::Cli;
use crate::engine::Engine;
use crate::output::output;
use crate::process_wrapper::Cmd;
use crate::sys::SysApi;
use crate::time_wrapper::Duration;
use crate::view::View;

#[instrument(level = "debug")]
pub async fn run<SI: SysApi>(cli: Cli, sys: SI) -> Result<()> {
    let refresh = Duration::milliseconds(250);
    let sleep = Duration::seconds(cli.period.into());
    let cmd = Cmd::from(cli.command.clone());
    let engine = Engine::new(
        sys.clone(),
        cmd.clone(),
        refresh,
        sleep,
        cli.until_success,
        cli.until_failure,
    )?;
    let view = View::new(cmd, refresh, sleep, engine);
    output(view).await
}
