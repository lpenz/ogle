// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use color_eyre::{Result, eyre::eyre};
use man::prelude::*;
use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::path;

fn generate_man_page<P: AsRef<path::Path>>(outdir: P) -> Result<()> {
    let outdir = outdir.as_ref();
    let man_path = outdir.join("ogle.1");
    let manpage = Manual::new("ogle")
        .about("Run a command-line periodically showing the output only when it changes")
        .author(Author::new("Leandro Lisboa Penz").email("lpenz@lpenz.org"))
        .flag(
            Flag::new()
                .short("-c")
                .long("--shell")
                .help("Invoke the shell on the single command argument"),
        )
        .flag(
            Flag::new()
                .short("-z")
                .long("--until-success")
                .help("Loop until the command exists with success"),
        )
        .option(
            Opt::new("period")
                .short("-p")
                .long("--period")
                .default_value("1")
                .help("Period to sleep between executions"),
        )
        .flag(
            Flag::new()
                .short("-h")
                .long("--help")
                .help("Prints help information"),
        )
        .flag(
            Flag::new()
                .short("-V")
                .long("--version")
                .help("Prints version information"),
        )
        .arg(Arg::new("COMMAND"))
        .arg(Arg::new("[ ARGS ]"))
        .description("ogle runs the provided command, optionally via the shell, and stores its output. After 'period' has passed, it runs the same command again, and only starts printing its output if it's different than the previous execution. It also prints a timestamp, and keeps a status line with run information.")
        .example(
            Example::new()
                .text("Monitor the current directory for changes")
                .command("ogle ls"),
        )
        .example(
            Example::new()
                .text("Show the current time every 3 seconds")
                .command("ogle -p 3 date"),
        )
        .example(
            Example::new()
                .text("Poor man's top using ps")
                .command("ogle --shell -- 'ps -eo %cpu,args --sort -%cpu | head'"),
        )
        .render();
    File::create(man_path)?.write_all(manpage.as_bytes())?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install()?;
    let mut outdir =
        path::PathBuf::from(env::var_os("OUT_DIR").ok_or_else(|| eyre!("error getting OUT_DIR"))?);
    fs::create_dir_all(&outdir)?;
    generate_man_page(&outdir)?;
    // build/ogle-*/out
    outdir.pop();
    // build/ogle-*
    outdir.pop();
    // build
    outdir.pop();
    // .
    // (either target/release or target/build)
    generate_man_page(&outdir)?;
    Ok(())
}
