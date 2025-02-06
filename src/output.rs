// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Wrapper for "low-level" system function used for the output
//!
//! This implements a command design pattern, which makes it very easy
//! to test that we are issuing the correct commands.

use color_eyre::Result;
use enum_dispatch::enum_dispatch;
use tokio_stream::Stream;
use tokio_stream::StreamExt;

use crate::term_wrapper::*;

#[enum_dispatch(OutputCommand)]
pub trait OutputCommandTrait {
    fn execute(&self);
}

pub struct MoveCursorUp(pub usize);
impl OutputCommandTrait for MoveCursorUp {
    fn execute(&self) {
        move_cursor_up(self.0).unwrap()
    }
}

pub struct ClearLine {}
impl OutputCommandTrait for ClearLine {
    fn execute(&self) {
        clear_line().unwrap()
    }
}

pub struct WriteAll(pub Vec<u8>);
impl OutputCommandTrait for WriteAll {
    fn execute(&self) {
        write_all(&self.0).unwrap()
    }
}

#[enum_dispatch]
pub enum OutputCommand {
    MoveCursorUp,
    ClearLine,
    WriteAll,
}

/// This function runs all commands in the provided stream until it is
/// exhausted.
#[allow(dead_code)]
pub async fn output<S>(mut stream: S) -> Result<()>
where
    S: Stream<Item = OutputCommand> + std::marker::Unpin,
{
    while let Some(cmd) = stream.next().await {
        cmd.execute();
    }
    Ok(())
}
