// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Wrapper for user interaction.
//!
//! For now we just check if the user has typed ENTER, which makes
//! ogle exit after the current run is over.

use color_eyre::Report;
use color_eyre::eyre::eyre;
use crossterm::tty::IsTty;
use crossterm::{
    event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io;
use tokio_stream::Stream;
use tracing::info;
use tracing::instrument;

/// An event coming from the user
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserEvent {
    /// Let the current execution finish and exit instead of sleeping.
    Quit,
    /// Kill the underlying program immediately and exit.
    Kill,
}

impl TryFrom<KeyEvent> for UserEvent {
    type Error = Report;
    fn try_from(ke: KeyEvent) -> Result<Self, Self::Error> {
        if ke.code == KeyCode::Char('q')
            || (ke.code == KeyCode::Char('d') && ke.modifiers == KeyModifiers::CONTROL)
        {
            Ok(UserEvent::Quit)
        } else if ke.code == KeyCode::Char('k')
            || (ke.code == KeyCode::Char('c') && ke.modifiers == KeyModifiers::CONTROL)
        {
            Ok(UserEvent::Kill)
        } else {
            Err(eyre!("unrecognized key event {:?}", ke))
        }
    }
}

impl TryFrom<&Event> for UserEvent {
    type Error = Report;
    fn try_from(event: &Event) -> Result<Self, Self::Error> {
        match event {
            Event::Key(ke) => UserEvent::try_from(*ke),
            event => Err(eyre!("unrecognized event {:?}", event)),
        }
    }
}

/// A wrapper for `crossterm` that polls the keyboard and provides the
/// keypress in a tokio stream.
///
/// Also provides a virtual implementation for use in tests.
#[derive(Debug, Default)]
pub enum UserStream {
    /// A real implementation that gets KeyEvents from an EventStream
    Real(EventStream),
    /// A virtual implementation that doesn't do anything.
    #[default]
    Virtual,
}

impl UserStream {
    pub fn new_real() -> Option<UserStream> {
        let stdin = io::stdin();
        if stdin.is_tty() {
            let _ = enable_raw_mode();
            Some(UserStream::Real(EventStream::new()))
        } else {
            None
        }
    }

    pub fn new_virtual() -> UserStream {
        UserStream::Virtual
    }
}

impl Drop for UserStream {
    fn drop(&mut self) {
        if matches!(self, UserStream::Real(_)) {
            let _ = disable_raw_mode();
        }
    }
}

impl Stream for UserStream {
    type Item = UserEvent;

    #[instrument(level = "debug", ret, skip(cx))]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        match this {
            UserStream::Real(event_stream) => {
                let next = Pin::new(event_stream).poll_next(cx);
                match next {
                    Poll::Ready(Some(Ok(event))) => match UserEvent::try_from(&event) {
                        Ok(user_event) => Poll::Ready(Some(user_event)),
                        Err(e) => {
                            info!(
                                event = ?event,
                                error=%e,
                                "error converting Event into UserEvent"
                            );
                            Poll::Pending
                        }
                    },
                    Poll::Ready(Some(Err(error))) => {
                        info!(
                            error = %error,
                            "EventStream yielded an error"
                        );
                        Poll::Pending
                    }
                    Poll::Ready(None) => Poll::Ready(None),
                    Poll::Pending => Poll::Pending,
                }
            }
            UserStream::Virtual => Poll::Ready(None),
        }
    }
}
