// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

/// A periodic stream that can be finished
/// This is heavily based on tokio-stream::IntervalStream
/// We are not using IntervalStream itself due to Pin issues.
use core::pin::Pin;
use core::task::Context;
use core::task::Poll;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::time;
use tokio_stream::Stream;

#[cfg(test)]
use tokio_stream::StreamExt; // for next()

#[derive(Default)]
pub struct Ticker {
    closed: Arc<Mutex<bool>>,
}

impl Ticker {
    pub fn create_stream(&self, period: time::Duration) -> TickerStream {
        TickerStream {
            interval: time::interval(period),
            closed: Arc::clone(&self.closed),
        }
    }

    pub fn close(&self) {
        let mut closed = self.closed.lock().unwrap();
        *closed = true;
    }
}

pub struct TickerStream {
    interval: time::Interval,
    closed: Arc<Mutex<bool>>,
}

impl Stream for TickerStream {
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<()>> {
        let closed = *self.closed.lock().unwrap();
        if closed {
            Poll::Ready(None)
        } else {
            self.interval.poll_tick(cx).map(|_| Some(()))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (std::usize::MAX, None)
    }
}

#[tokio::test]
async fn ticker_basic() {
    let ticker = Ticker::default();
    let mut ts = ticker.create_stream(time::Duration::from_millis(250));
    ts.next().await; // First is right away
    let now0 = time::Instant::now();
    for _ in 0..4 {
        let now = time::Instant::now();
        ts.next().await;
        let elapsed = time::Instant::now() - now;
        assert!((elapsed.as_millis() as i32 - 250_i32).abs() < 10);
    }
    let elapsed = time::Instant::now() - now0;
    assert!((elapsed.as_millis() as i32 - 1000_i32).abs() < 10);
    ticker.close();
    let now = time::Instant::now();
    for _ in 0..4 {
        ts.next().await;
    }
    let elapsed = time::Instant::now() - now;
    assert!(elapsed.as_millis() < 10);
}
