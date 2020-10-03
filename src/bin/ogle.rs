// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

fn main() {
    let args = ogle::cli::Cli::from_args();
    ogle::runner::run(&args);
}
