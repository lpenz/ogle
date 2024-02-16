// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use clap::Parser;

#[cfg(test)]
use color_eyre::Result;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Period to sleep between executions
    #[arg(short, long, default_value = "1")]
    pub period: u64,

    /// Loop until the command exists with success
    #[arg(short = 'z', long = "until-success")]
    pub until_success: bool,

    /// The command to run
    #[clap(value_parser, required = true)]
    pub command: Vec<String>,
}

#[test]
fn empty() {
    let cli = Cli::try_parse_from(vec!["ogle"]);
    assert!(cli.is_err());
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
