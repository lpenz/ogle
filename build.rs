// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use clap::CommandFactory;
use clap_complete::generate_to;
use clap_complete::shells::Bash;
use clap_complete::shells::Fish;
use clap_complete::shells::Zsh;
use color_eyre::{Result, eyre::eyre};
use man::prelude::*;
use std::env;
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path;

include!("src/cli.rs");

fn generate_man_page<P: AsRef<path::Path>>(
    cmd: &clap::Command,
    name: &str,
    outdir: P,
) -> Result<()> {
    let outdir = outdir.as_ref();
    let man_path = outdir.join(format!("{}.1", name));
    let manpage: Manual = clap2man::Manual::try_from(cmd)
        .map_err(|e| eyre!(e))?
        .into();
    let manpage = manpage
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
                .text("Try to connect to an ssh server until it succeeds")
                .command("ogle -z ssh server true"),
        )
        .example(
            Example::new()
                .text("Poor man's top using ps")
                .command("ogle -- /bin/bash -c 'ps -eo %cpu,args --sort -%cpu | head'"),
        );
    std::fs::write(man_path, manpage.render())?;
    Ok(())
}

/// Return the profile directory (e.g. `target/release`) that contains OUT_DIR.
///
/// Cargo no longer adds a hash suffix to the build script directory, so its
/// depth is not stable. Find it by walking up from OUT_DIR instead of popping
/// a fixed number of components.
fn profile_dir(outdir: &path::Path, profile: &str) -> Result<path::PathBuf> {
    outdir
        .ancestors()
        .find(|dir| dir.file_name().is_some_and(|name| name == profile))
        .map(|dir| dir.to_path_buf())
        .ok_or_else(|| eyre!("could not find {profile:?} directory in OUT_DIR {outdir:?}"))
}

fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install()?;
    let cmd = Cli::command();
    let name = cmd
        .get_display_name()
        .unwrap_or_else(|| cmd.get_name())
        .to_owned();
    let outdir =
        path::PathBuf::from(env::var_os("OUT_DIR").ok_or_else(|| eyre!("error getting OUT_DIR"))?);
    fs::create_dir_all(&outdir)?;
    generate_man_page(&cmd, &name, &outdir)?;
    // Generate assets next to the binary, where the packaging metadata expects them.
    let outdir = profile_dir(&outdir, &env::var("PROFILE")?)?;
    generate_man_page(&cmd, &name, &outdir)?;
    // Generate shell completions:
    generate_to(Bash, &mut cmd.clone(), &name, &outdir)?;
    let path = generate_to(Fish, &mut cmd.clone(), &name, &outdir)?;
    let mut fd = OpenOptions::new().append(true).open(path)?;
    writeln!(fd, "complete -c ogle --wraps command")?;
    writeln!(fd, "complete -c ogle --no-files")?;
    generate_to(Zsh, &mut cmd.clone(), &name, &outdir)?;
    Ok(())
}
