// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use std::env;
use std::ffi::OsString;
use std::process;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Cli {
    /// Period to sleep between executions
    #[structopt(short, long, default_value = "1")]
    pub period: u64,

    /// Invoke the shell on the single command argument
    #[structopt(short = "c", long = "shell")]
    pub shell: bool,

    /// The command to run
    pub command: Vec<String>,
}

impl Cli {
    pub fn from_args() -> Cli {
        Cli::from_iter_safe(&mut env::args_os()).unwrap_or_else(|e| {
            eprintln!("{}", e.message);
            process::exit(1);
        })
    }

    pub fn from_iter_safe<I>(iter: I) -> Result<Cli, clap::Error>
    where
        Self: Sized,
        I: IntoIterator,
        I::Item: Into<OsString> + Clone,
    {
        let clap = Cli::clap().get_matches_from_safe(iter)?;
        let cli = Cli::from_clap(&clap);
        if cli.command.is_empty() {
            Err(clap::Error::with_description(
                "No command specified

For more information try --help",
                clap::ErrorKind::EmptyValue,
            ))
        } else if cli.shell && cli.command.len() != 1 {
            Err(clap::Error::with_description(
                "In shell mode, command must be in a single argument

For more information try --help",
                clap::ErrorKind::InvalidValue,
            ))
        } else {
            Ok(cli)
        }
    }
}

#[test]
fn empty() {
    let cli = Cli::from_iter_safe(vec!["ogle"]);
    assert!(cli.is_err());
}

#[test]
fn dashes() {
    let cli = Cli::from_iter_safe(vec!["ogle", "--"]);
    assert!(cli.is_err());
    let cli = Cli::from_iter_safe(vec!["ogle", "--", "ls", "-l"]).unwrap();
    assert_eq!(cli.command[0], "ls");
    assert_eq!(cli.command[1], "-l");
    assert_eq!(cli.command.len(), 2);
    assert_eq!(cli.period, 1);
}

#[test]
fn period() {
    let cli = Cli::from_iter_safe(vec!["ogle", "-p", "5", "--", "ls", "-l"]).unwrap();
    assert_eq!(cli.period, 5);
    let cli = Cli::from_iter_safe(vec!["ogle", "--period", "7", "--", "ls", "-l"]).unwrap();
    assert_eq!(cli.period, 7);
}

#[test]
fn shell() {
    let cli = Cli::from_iter_safe(vec!["ogle", "-c", "--", "ls", "-l"]);
    assert!(cli.is_err());
    let cli = Cli::from_iter_safe(vec!["ogle", "-c", "--", "ls -l"]).unwrap();
    assert!(cli.shell);
    assert_eq!(cli.command[0], "ls -l");
    assert_eq!(cli.command.len(), 1);
}
