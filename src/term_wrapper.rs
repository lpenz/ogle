// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use console::Term;
use std::io::Result;
use std::io::Write;
use std::sync::LazyLock;
use std::sync::Mutex;

static TERM: LazyLock<Mutex<Term>> = LazyLock::new(|| Mutex::new(Term::stdout()));

pub fn size_checked() -> Option<(u16, u16)> {
    TERM.lock().unwrap().size_checked()
}

pub fn move_cursor_up(n: usize) -> Result<()> {
    TERM.lock().unwrap().move_cursor_up(n)
}

pub fn clear_line() -> Result<()> {
    TERM.lock().unwrap().clear_line()
}

pub fn write_all(buf: &[u8]) -> Result<()> {
    TERM.lock().unwrap().write_all(buf)
}

pub fn flush() -> Result<()> {
    TERM.lock().unwrap().flush()
}
