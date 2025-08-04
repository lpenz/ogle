// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use std::collections::VecDeque;
use tracing::instrument;

#[derive(Debug, Default)]
pub struct Differ {
    changed: bool,
    lines: VecDeque<String>,
    iline: usize,
}

impl Differ {
    #[instrument(level = "debug")]
    pub fn reset(&mut self) {
        self.changed = false;
        self.iline = 0;
    }

    #[instrument(level = "debug", skip(self), fields(iline=self.iline, line=line))]
    pub fn push(&mut self, line: String) {
        if self.changed {
            // If we are in "changed" mode, we just append the lines:
            self.lines.push_back(line);
        } else {
            // Otherwise, we check
            if self.iline >= self.lines.len() || self.lines.get(self.iline) != Some(&line) {
                // New or different line.
                self.changed = true;
                self.lines.truncate(self.iline);
                self.lines.push_back(line);
                // self.iline is now the read position, which starts at 0:
                self.iline = 0;
            } else {
                // Same line, get ready for the next one:
                self.iline += 1;
            }
        }
    }

    #[instrument(level = "debug", skip(self), fields(changed=self.changed))]
    pub fn has_changed(&self) -> bool {
        self.changed
    }
}

impl Iterator for &mut Differ {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        assert!(self.changed, "can only iterate over a changed Differ");
        if self.iline == self.lines.len() {
            None
        } else {
            let i = self.iline;
            self.iline += 1;
            Some(self.lines[i].clone())
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_basic() {
        let mut d = Differ::default();
        assert!(!d.has_changed());
        d.push("1".to_owned());
        assert!(d.has_changed());
        d.push("2".to_owned());
        assert_eq!(d.collect::<Vec<_>>(), vec!["1", "2"]);
        d.reset();
        assert!(!d.has_changed());
        d.push("1".to_owned());
        assert!(!d.has_changed());
        d.push("2".to_owned());
        assert!(!d.has_changed());
        d.push("3".to_owned());
        assert!(d.has_changed());
        assert_eq!(d.collect::<Vec<_>>(), vec!["1", "2", "3"]);
    }
}
