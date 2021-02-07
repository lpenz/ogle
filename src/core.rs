// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

#[derive(Debug)]
pub enum StreamItem {
    Line(String),
    Err(anyhow::Error),
    Tick,
}
