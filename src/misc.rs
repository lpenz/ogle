// Copyright (C) 2024 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

pub fn localnow() -> String {
    format!(
        "{}",
        chrono::offset::Local::now().format("%Y-%m-%d %H:%M:%S")
    )
}
