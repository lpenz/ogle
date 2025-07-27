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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time_wrapper::Instant;
    use color_eyre::Result;

    #[test]
    fn test_ofmt_helper() -> Result<()> {
        let timestamp = Instant::default();
        assert_eq!(
            ofmt_helper(&timestamp, "test line"),
            format!("<O> {} test line", timestamp)
        );
        Ok(())
    }

    #[test]
    fn test_ofmt_macro() -> Result<()> {
        let timestamp = Instant::default();
        let formatted = ofmt!(&timestamp, "hello {}", 123);
        assert_eq!(formatted, format!("<O> {} hello 123", timestamp));
        Ok(())
    }

    #[test]
    fn test_ofmt_timeless_helper() -> Result<()> {
        assert_eq!(ofmt_timeless_helper("timeless test"), "<O> timeless test");
        Ok(())
    }

    #[test]
    fn test_ofmt_timeless_macro() -> Result<()> {
        let formatted = ofmt_timeless!("value: {}", 42.5);
        assert_eq!(formatted, "<O> value: 42.5");
        Ok(())
    }
}
