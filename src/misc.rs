// Copyright (C) 2024 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use color_eyre::Result;
use console::Term;

use crate::timewrap::Instant;

pub fn term_width(term: &Term) -> usize {
    if let Some((_, w)) = term.size_checked() {
        w as usize
    } else {
        80
    }
}

pub fn term_clear_line(term: &Term) -> Result<()> {
    term.move_cursor_up(1)?;
    term.clear_line()?;
    Ok(())
}

pub fn ofmt_helper(timestamp: &Instant, line: &str) -> String {
    format!("<O> {} {}", timestamp, line)
}

macro_rules! ofmt {
    ($timestamp: expr, $($t:tt)*) => {{
        crate::misc::ofmt_helper($timestamp, &format!($($t)*))
    }};
}
