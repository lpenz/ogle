// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Wrapper for low-level terminal manipulation.
//!
//! This wraps [`crossterm`] at the moment.

use crossterm::{
    cursor::MoveUp,
    execute,
    terminal::{Clear, ClearType, size},
};
use std::io::Result;
use std::io::{Write, stdout};

/// Returns the width of the terminal
///
/// Uses [`crossterm::terminal::size`]
#[allow(dead_code)]
pub fn get_width() -> Option<u16> {
    size().ok().map(|(w, _)| w)
}

/// Move the cursor up by `n` lines, if possible.
///
/// Wraps [`crossterm::cursor::MoveUp`]
pub fn move_cursor_up(n: u16) -> Result<()> {
    execute!(stdout(), MoveUp(n))
}

/// Clear the current line.
///
/// Position the cursor at the beginning of the current line.
///
/// Wraps [`crossterm::terminal::Clear`]
pub fn clear_line() -> Result<()> {
    execute!(stdout(), Clear(ClearType::CurrentLine))
}

/// Attempts to write an entire buffer to the terminal.
///
/// Wraps regular io functions but also moves the cursor to the first
/// column with [`crossterm::cursor::MoveToColumn`]
pub fn write_all(buf: &[u8]) -> Result<()> {
    stdout().write_all(buf)?;
    stdout().flush()?;
    Ok(())
}
