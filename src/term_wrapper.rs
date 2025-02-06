// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Wrapper for low-level terminal manipulation.
//!
//! This wraps [`console::Term`] at the moment, and provides singleton
//! functions that do not require an object. We do that by wrapping
//! the Term object in a [`std::sync::Mutex`].
//!
//! A side-effect of this style of interface is that we can change the
//! underlying library with (hopefully) no impact on users.

use console::Term;
use std::io::Result;
use std::io::Write;
use std::sync::LazyLock;
use std::sync::Mutex;

static TERM: LazyLock<Mutex<Term>> = LazyLock::new(|| Mutex::new(Term::stdout()));

/// Returns the width of the terminal
///
/// Uses [`console::Term::size_checked`]
pub fn get_width() -> Option<u16> {
    TERM.lock()
        .expect("unable to lock TERM")
        .size_checked()
        .map(|(_, width)| width)
}

/// Move the cursor up by `n` lines, if possible.
///
/// Wraps [`console::Term::move_cursor_up`]
pub fn move_cursor_up(n: usize) -> Result<()> {
    TERM.lock().expect("unable to lock TERM").move_cursor_up(n)
}

/// Clear the current line.
///
/// Position the cursor at the beginning of the current line.
///
/// Wraps [`console::Term::clear_line`]
pub fn clear_line() -> Result<()> {
    TERM.lock().expect("unable to lock TERM").clear_line()
}

/// Attempts to write an entire buffer to the terminal.
///
/// Wraps [`console::Term::write_all`]
pub fn write_all(buf: &[u8]) -> Result<()> {
    TERM.lock().expect("unable to lock TERM").write_all(buf)
}
