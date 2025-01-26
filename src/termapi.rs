// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

// Useful functions:

#[derive(Debug)]
pub struct TermWrapper {
    term: Term,
}

impl TermWrapper {
    fn width(&self) -> usize {
        if let Some((_, w)) = self.term.size_checked() {
            w as usize
        } else {
            80
        }
    }

    pub fn clear_line(&mut self) -> Result<()> {
        self.term.move_cursor_up(1)?;
        self.term.clear_line()?;
    }
}
