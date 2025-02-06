// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Module that wraps system functions used as inputs.
//!
//! Wrapping this makes it very easy to test the whole program. The
//! strategy is to parameterize the types with the [`SysApi`] trait,
//! and then instantiate them with [`SysReal`] in production, and with
//! [`SysVirtual`] in the tests. `SysVirtual` ends up being
//! essentially a mock object.

use color_eyre::Result;
use std::cell::RefCell;
use std::collections::VecDeque;
use tokio::process::Command;
use tokio_process_stream as tps;

use crate::process_wrapper::Cmd;
use crate::process_wrapper::Item;
use crate::process_wrapper::ProcessStream;
use crate::term_wrapper;
use crate::time_wrapper::Duration;
use crate::time_wrapper::Instant;

// SysApi ////////////////////////////////////////////////////////////

/// Trait providing the system functions available for mocking.
///
/// Users should depend on this type and then take [`SysVirtual`] in
/// tests or [`SysReal`] in production.
pub trait SysApi: std::fmt::Debug + Clone + Default {
    /// Returns an [`Instant`] that corresponds to the current
    /// wall-clock time.
    fn now(&self) -> Instant;

    /// Returns the width of the terminal.
    #[allow(dead_code)]
    fn get_width(&self) -> Option<u16>;

    /// Starts the execution of the provided [`Cmd`] and returns the
    /// corresponding [`ProcessStream`] object.
    ///
    /// The `ProcessStream` object yields each line printed by the
    /// command in its `stdout` and `stderr` via a stream. The same
    /// stream gets the [`ExitStatus`](std::process::ExitStatus) when
    /// the process finishes.
    fn run_command(&mut self, command: Cmd) -> Result<ProcessStream, std::io::Error>;
}

// SysReal ///////////////////////////////////////////////////////////

/// `SysReal` implements [`SysApi`] by calling real system functions.
#[derive(Debug, Clone, Default)]
pub struct SysReal {}

impl SysApi for SysReal {
    fn now(&self) -> Instant {
        Instant::from(chrono::offset::Utc::now())
    }
    fn get_width(&self) -> Option<u16> {
        term_wrapper::get_width()
    }
    fn run_command(&mut self, cmd: Cmd) -> Result<ProcessStream, std::io::Error> {
        let process_stream = tps::ProcessLineStream::try_from(Command::from(&cmd))?;
        Ok(ProcessStream::from(process_stream))
    }
}

// SysVirtual ////////////////////////////////////////////////////////

/// `SysVirtual` implements [`SysApi`] with behaviors appropriate for
/// testing.
///
/// Namely:
/// - [`SysVirtual::now`] starts at the
///   [epoch](chrono::DateTime::UNIX_EPOCH) and increments its return
///   value by 1 second at every call.
/// - [`SysVirtual::get_width`] always returns 80.
/// - [`SysVirtual::run_command`] ignores the `cmd` argument and
///   yields items from a list that was provided to
///   [`SysVirtual::set_items`].
#[derive(Debug, Clone, Default)]
pub struct SysVirtual {
    now: RefCell<Instant>,
    items: VecDeque<Item>,
}

impl SysApi for SysVirtual {
    /// Returns a "fake" current time by starting at the
    /// [epoch](chrono::DateTime::UNIX_EPOCH) and incrementing the
    /// return value by 1 second at every call.
    fn now(&self) -> Instant {
        let mut now_ref = self.now.borrow_mut();
        let now = *now_ref;
        *now_ref = &now + &Duration::seconds(1);
        now
    }
    fn get_width(&self) -> Option<u16> {
        Some(80)
    }
    /// Yields items from the list that was provided to
    /// [`SysVirtual::set_items`].
    ///
    /// The `cmd` argument is not used.
    fn run_command(&mut self, _cmd: Cmd) -> Result<ProcessStream, std::io::Error> {
        let items = std::mem::take(&mut self.items);
        Ok(ProcessStream::from(items))
    }
}

impl SysVirtual {
    /// Sets the list that is going to be yielded by the stream
    /// returned by [`SysVirtual::run_command`].
    #[allow(dead_code)]
    pub fn set_items(&mut self, items: Vec<Item>) {
        self.items = items.into_iter().collect();
    }
}

// Tests /////////////////////////////////////////////////////////////

#[cfg(test)]
pub mod test {
    use color_eyre::Result;
    use tokio_stream::StreamExt;

    use crate::process_wrapper::ExitSts;
    use crate::sys::SysReal;
    use crate::sys::SysVirtual;

    use super::*;

    // A simple test for SysVirtual as we cover it better in
    // downstream tests

    #[tokio::test]
    async fn test_sysvirtual() -> Result<()> {
        let list = vec![
            Item::Stdout("stdout".into()),
            Item::Stderr("stderr".into()),
            Item::Done(Ok(ExitSts::default())),
        ];
        let mut sys = SysVirtual::default();
        sys.set_items(list.clone());
        let cmd = Cmd::default();
        assert_eq!(format!("{}", cmd), "");
        let streamer = sys.run_command(cmd)?;
        assert_eq!(format!("{:?}", streamer), "ProcessStream::Virtual");
        let streamed = streamer.collect::<Vec<_>>().await;
        assert_eq!(streamed, list);
        assert_eq!(sys.now(), Instant::default());
        assert_eq!(sys.now(), &Instant::default() + &Duration::seconds(1));
        assert_eq!(sys.get_width(), Some(80));
        Ok(())
    }

    // A couple of tests for SysReal for minimal coverage

    #[test]
    fn test_sysreal_now() {
        let sys = SysReal::default();
        let now = sys.now();
        let now2 = sys.now();
        assert!(&now2 >= &now);
    }

    #[tokio::test]
    async fn test_sysreal_run_command() -> Result<()> {
        let mut sys = SysReal::default();
        let cmdarr = ["true"];
        let cmd = Cmd::from(&cmdarr[..]);
        let streamer = sys.run_command(cmd)?;
        let streamed = streamer.collect::<Vec<_>>().await;
        assert_eq!(streamed, vec![Item::Done(Ok(ExitSts::default())),]);
        Ok(())
    }
}
