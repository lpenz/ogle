// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

// ogle's CLI using [`clap`]
//
// This is not a module-level doc because we `include!` it in build.rs.
//
// [`clap`]: https://docs.rs/clap/latest/clap/

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Period to sleep between executions
    #[arg(short, long, default_value = "1")]
    pub period: u32,

    /// Loop until the command exists with success
    #[arg(short = 'z', long = "until-success")]
    pub until_success: bool,

    /// Loop until the command exists with a failure
    #[arg(short = 'e', long = "until-failure")]
    pub until_failure: bool,

    /// The command to run
    #[clap(value_parser, required = true)]
    pub command: Vec<String>,
}

#[cfg(test)]
mod tests {
    use crate::process_wrapper::Cmd;
    use color_eyre::Result;
    use color_eyre::eyre::WrapErr;
    use std::process::ExitStatus;

    use super::*;

    #[test]
    fn empty() {
        let cli = Cli::try_parse_from(vec!["ogle"]);
        assert!(cli.is_err(), "should require at least a command to run");
    }

    #[test]
    fn dashes() -> Result<()> {
        let cli = Cli::try_parse_from(vec!["ogle", "--"]);
        assert!(cli.is_err());
        let cli = Cli::try_parse_from(vec!["ogle", "--", "ls", "-l"])?;
        assert_eq!(cli.command[0], "ls");
        assert_eq!(cli.command[1], "-l");
        assert_eq!(cli.command.len(), 2);
        assert_eq!(cli.period, 1);
        Ok(())
    }

    #[test]
    fn period() -> Result<()> {
        let cli = Cli::try_parse_from(vec!["ogle", "-p", "5", "--", "ls", "-l"])?;
        assert_eq!(cli.period, 5);
        let cli = Cli::try_parse_from(vec!["ogle", "--period", "7", "--", "ls", "-l"])?;
        assert_eq!(cli.period, 7);
        Ok(())
    }

    #[test]
    fn until() -> Result<()> {
        let cli = Cli::try_parse_from(vec!["ogle", "-z", "--", "true"])?;
        assert!(cli.until_success);
        assert!(!cli.until_failure);
        let cli = Cli::try_parse_from(vec!["ogle", "-e", "--", "true"])?;
        assert!(!cli.until_success);
        assert!(cli.until_failure);
        Ok(())
    }

    async fn run_cmd(cmd: Vec<&str>) -> Result<ExitStatus> {
        let cli = Cli::try_parse_from(cmd)?;
        let cmd = Cmd::from(cli.command.clone());
        let mut cmd = tokio::process::Command::from(&cmd);
        cmd.spawn()?.wait().await.wrap_err("")
    }

    #[tokio::test]
    async fn get_cmd_command() -> Result<()> {
        let exit = run_cmd(vec!["ogle", "true"]).await?;
        assert!(exit.success());
        let exit = run_cmd(vec!["ogle", "false"]).await?;
        assert!(!exit.success());
        Ok(())
    }
}
