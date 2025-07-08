// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Wrapper for user interaction.
//!
//! For now we just check if the user has typed ENTER, which makes
//! ogle exit after the current run is over.

use is_terminal::IsTerminal;
use pin_project::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::Stdin;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio_stream::Stream;
use tokio_stream::wrappers::LinesStream;
use tracing::instrument;

/// A wrapper for [`tokio_stream::wrappers::LinesStream`] with an
/// inner [`tokio::io::Stdin`].
///
/// Also provides a virtual implementation for use in tests.
#[pin_project(project = UserStreamProj)]
#[derive(Debug, Default)]
pub enum UserStream {
    /// A real implementation that reads a line from stdin.
    Real(LinesStream<BufReader<Stdin>>),
    /// A virtual implementation that doesn't do anything.
    #[default]
    Virtual,
}

impl UserStream {
    pub fn new_real() -> Option<UserStream> {
        let stdin = io::stdin();
        if stdin.is_terminal() {
            let bufstdin = BufReader::new(stdin);
            let linesstream = LinesStream::new(bufstdin.lines());
            Some(UserStream::Real(linesstream))
        } else {
            None
        }
    }

    pub fn new_virtual() -> UserStream {
        UserStream::Virtual
    }
}

impl Stream for UserStream {
    type Item = String;

    #[instrument(level = "debug", ret, skip(cx))]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match this {
            UserStreamProj::Real(linesstream) => {
                let next = Pin::new(linesstream).poll_next(cx);
                match next {
                    Poll::Ready(Some(Ok(s))) => Poll::Ready(Some(s)),
                    Poll::Ready(None) => Poll::Ready(None),
                    Poll::Pending => Poll::Pending,
                    // End stream in case of error:
                    Poll::Ready(_) => Poll::Ready(None),
                }
            }
            UserStreamProj::Virtual {} => Poll::Ready(None),
        }
    }
}
