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
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io;
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};
use tokio_stream::Stream;
use tracing::info;
use tracing::instrument;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserEvent {
    Kill,
    Quit,
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

/// A wrapper for `crossterm` that polls the keyboard and provides the
/// keypress in a tokio stream.
///
/// Also provides a virtual implementation for use in tests.
#[derive(Debug, Default)]
pub enum UserStream {
    /// A real implementation that reads a line from stdin.
    Real(mpsc::UnboundedReceiver<UserEvent>),
    /// A virtual implementation that doesn't do anything.
    #[default]
    Virtual,
}

impl UserStream {
    pub fn new_real() -> Option<UserStream> {
        let stdin = io::stdin();
        if stdin.is_tty() {
            let (tx, rx) = mpsc::unbounded_channel::<UserEvent>();
            let _ = enable_raw_mode();
            tokio::spawn(async move {
                loop {
                    let key_event = matches!(event::poll(Duration::from_secs(0)), Ok(true))
                        .then(|| match event::read() {
                            Ok(Event::Key(key_event)) => Some(key_event),
                            Ok(_) => None,
                            Err(e) => {
                                panic!("could not read key after poll returned true: {}", e)
                            }
                        })
                        .flatten();
                    if let Some(key_event) = key_event {
                        // If sending the key fails, it's probably because we
                        // are in the process of being dropped, so we can
                        // ignore it:
                        match UserEvent::try_from(key_event) {
                            Ok(user_event) => {
                                let _ = tx.send(user_event);
                            }
                            Err(e) => {
                                info!(
                                    key_event = ?key_event,
                                    error=%e,
                                    "error converting KeyEvent into UserEvent"
                                );
                            }
                        }
                    } else {
                        // We tokio-sleep here to provide a cancellation point:
                        sleep(Duration::from_millis(127)).await;
                    }
                }
            });
            Some(UserStream::Real(rx))
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
            UserStream::Real(rx) => {
                let next = Pin::new(rx).poll_recv(cx);
                match next {
                    Poll::Ready(Some(ue)) => Poll::Ready(Some(ue)),
                    Poll::Ready(None) => Poll::Ready(None),
                    Poll::Pending => Poll::Pending,
                }
            }
            UserStream::Virtual => Poll::Ready(None),
        }
    }
}
