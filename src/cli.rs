// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! ogle's CLI using [`clap`]
//!
//! [`clap`]: https://docs.rs/clap/latest/clap/

use clap::Parser;

use crate::process_wrapper::Cmd;

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

impl Cli {
    pub fn get_cmd(&self) -> Cmd {
        Cmd::from(self.command.clone())
    }
}

#[cfg(test)]
mod tests {
    use color_eyre::Result;

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

    #[tokio::test]
    async fn get_cmd_command() -> Result<()> {
        let cli = Cli::try_parse_from(vec!["ogle", "true"])?;
        let mut cmd = tokio::process::Command::from(&cli.get_cmd());
        let exit = cmd.spawn()?.wait().await?;
        assert!(exit.success());
        let cli = Cli::try_parse_from(vec!["ogle", "false"])?;
        let mut cmd = tokio::process::Command::from(&cli.get_cmd());
        let exit = cmd.spawn()?.wait().await?;
        assert!(!exit.success());
        Ok(())
    }
}
