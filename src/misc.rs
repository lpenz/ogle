// Copyright (C) 2024 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use crate::time_wrapper::Instant;

pub fn ofmt_helper(timestamp: &Instant, line: &str) -> String {
    format!("<O> {timestamp} {line}")
}

macro_rules! ofmt {
    ($timestamp: expr, $($t:tt)*) => {{
        crate::misc::ofmt_helper($timestamp, &format!($($t)*))
    }};
}
