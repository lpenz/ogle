// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Cli {
    /// Period to sleep between executions
    #[structopt(short = "p", long = "period", default_value = "1")]
    pub period: u64,

    /// The command to run
    pub command: Vec<String>,
}

#[test]
fn dashes() {
    let cli = Cli::from_iter(vec!["ogle", "--", "ls", "-l"]);
    assert_eq!(cli.command[0], "ls");
    assert_eq!(cli.command[1], "-l");
    assert_eq!(cli.command.len(), 2);
    assert_eq!(cli.period, 1);
}

#[test]
fn period() {
    let cli = Cli::from_iter(vec!["ogle", "-p", "5", "--", "ls", "-l"]);
    assert_eq!(cli.period, 5);
    let cli = Cli::from_iter(vec!["ogle", "--period", "7", "--", "ls", "-l"]);
    assert_eq!(cli.period, 7);
}
