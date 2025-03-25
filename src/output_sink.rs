// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Wrapper for "low-level" system function used for the output
//!
//! This implements a command design pattern, which makes it very easy
//! to test that we are issuing the correct commands.

use enum_dispatch::enum_dispatch;
use tokio_stream::Stream;
use tokio_stream::StreamExt;

use crate::term_wrapper::*;

#[enum_dispatch(OutputCommand)]
pub trait OutputCommandTrait {
    fn execute(&self);
}

pub struct MoveCursorUp(usize);
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

pub struct WriteAll(Vec<u8>);
impl OutputCommandTrait for WriteAll {
    fn execute(&self) {
        write_all(&self.0).unwrap()
    }
}

pub struct Flush();
impl OutputCommandTrait for Flush {
    fn execute(&self) {
        flush().unwrap()
    }
}

#[enum_dispatch]
pub enum OutputCommand {
    MoveCursorUp,
    ClearLine,
    WriteAll,
    Flush,
}

/// This function runs all commands in the provided stream until it is
/// exhausted.
#[allow(dead_code)]
pub async fn output_sink<S>(mut stream: S)
where
    S: Stream<Item = OutputCommand> + std::marker::Unpin,
{
    while let Some(cmd) = stream.next().await {
        cmd.execute();
    }
}
