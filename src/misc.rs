// Copyright (C) 2024 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Misc utility functions

use crate::time_wrapper::Instant;

pub fn ofmt_helper(timestamp: &Instant, line: &str) -> String {
    format!("<O> {timestamp} {line}")
}

/// Print a message on stdout, with a timestamp, in the standard
/// `ogle` format
macro_rules! ofmt {
    ($timestamp: expr, $($t:tt)*) => {{
        crate::misc::ofmt_helper($timestamp, &format!($($t)*))
    }};
}

pub fn ofmt_timeless_helper(line: &str) -> String {
    format!("<O> {line}")
}

/// Print a message on stdout, without a timestamp, in the standard
/// `ogle` format
macro_rules! ofmt_timeless {
    ($($t:tt)*) => {{
        crate::misc::ofmt_timeless_helper(&format!($($t)*))
    }};
}
