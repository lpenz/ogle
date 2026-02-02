// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "console")]
    console_subscriber::init();
    ogle::main()
}
