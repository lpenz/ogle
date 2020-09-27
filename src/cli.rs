// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Cli {
    /// The command to run
    pub command: Vec<String>,
}

#[test]
fn dashes() {
    let cli = Cli::from_iter(vec!["ogle", "--", "ls", "-l"]);
    assert_eq!(cli.command[0], "ls");
    assert_eq!(cli.command[1], "-l");
    assert_eq!(cli.command.len(), 2);
}
