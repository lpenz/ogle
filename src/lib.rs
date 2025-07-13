// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![allow(rustdoc::private_intra_doc_links)]

//! **ogle** is a program that runs the given command-line periodically,
//! showing the output only when it is different than the last.
//!
//! The simplest way to show most of the features of *ogle* is by asking
//! it to run `date; sleep` in a shell, with a waiting period of 3s:
//!
//! ![demo](https://raw.githubusercontent.com/lpenz/ogle/main/demos/demo-sleep.gif)
//!
//! Lines that were written by ogle all start with `=>`. On the first
//! execution, ogle shows a spinner while the command is running. On the
//! next executions, ogle shows a progress bar, where the total
//! corresponds to the duration of the previous execution. The sleep time
//! is also shown, as a countdown. If the command returns an error to the
//! shell, the error value is displayed.
//!
//! ogle also supports limited interactive control with one-character
//! commands followed by ENTER:
//! - `q`: quit after when the process is no longer running.
//!
//! # Installation
//!
//! If you're a **Rust programmer**, ogle can be installed with `cargo`:
//!
//! ```bash
//! $ cargo install ogle
//! ```
//!
//! If you're a **Debian** user, ogle is available in
//! [packagecloud](https://packagecloud.io/app/lpenz/debian/search?q=ogle). Follow
//! these
//! [instruction](https://packagecloud.io/lpenz/debian/install#manual) to
//! use the package repository.
//!
//!
//! # Internals
//!
//! To make it fully testable, it uses a layered architecture based on
//! tokio streams which ends up being similar to how we use pipes in a
//! shell. We can divide it in the following layers:
//! - wrappers: we have 3 wrapper modules that abtract external
//!   libraries to provide us simpler types or types that provide that
//!   `impl` traits we need. They also make it easier to replate the
//!   underlying implementation in the future, if necessary. Namely:
//!   - [`process_wrapper`]: wraps process instantiation and I/O, and
//!     provides an [`Item`](process_wrapper::Item) that implements
//!     `Eq` so that we can use it in tests.
//!   - [`term_wrapper`]: implements terminal functions, mostly for
//!     output. As we are currently wrapping [`console`] and its
//!     functions require a [`console::Term`] object, we end up using
//!     a mutex here to abstract the singleton.
//!   - [`user_wrapper`]: abstract user interaction. At the moment, we
//!     just monitor `stdin` in line mode, and ogle exits gracefully
//!     when that's detected.
//!   - [`time_wrapper`]: home of the
//!     [`Instant`](time_wrapper::Instant) and
//!     [`Duration`](time_wrapper::Duration) types, which use types
//!     from [`chrono`] at the moment.
//! - [`sys`]: most of the ogle code doesn't really call functions
//!   that interact with the host system - we have the `sys` module
//!   for that. The module does that by providing a [`sys::SysApi`]
//!   trait that is then implemented by both the [`sys::SysReal`]
//!   type, which calls the system functions; and by the
//!   [`sys::SysVirtual`] type, which can be used to mock these calls
//!   in various ways.
//!
//! ```no_compile
//! sys -> engine -> view -> output
//! ```
//!
//! [watch (1)]: https://linux.die.net/man/1/watch
//!

use clap::Parser;
use std::error::Error;

#[macro_use]
mod misc;

mod cli;
mod differ;
mod orchestrator;
mod progbar;

mod process_wrapper;
mod term_wrapper;
mod time_wrapper;
mod user_wrapper;

mod sys;

mod engine;

mod view;

mod output;

/// Ogle main function, the single pub function in this lib.
#[tokio::main(flavor = "current_thread")]
pub async fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::ACTIVE)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let args = cli::Cli::parse();
    let sys = sys::SysReal::default();
    orchestrator::run(args, sys).await?;
    Ok(())
}
