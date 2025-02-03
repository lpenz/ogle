// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use color_eyre::Result;

use crate::time_wrapper::Duration;
use crate::time_wrapper::Instant;

// Basic functions:

pub fn progbar_sleeping(
    timestamp: &Instant,
    now: &Instant,
    start: &Instant,
    duration: &Duration,
) -> Result<String> {
    let msg = if duration.num_seconds() > 1 {
        let end = start + duration;
        let left = &end - now;
        ofmt!(timestamp, "sleeping for {}s", left.num_seconds() + 1)
    } else {
        ofmt!(timestamp, "sleeping")
    };
    Ok(msg)
}

pub fn progbar_running(
    width: usize,
    timestamp: &Instant,
    now: &Instant,
    start: &Instant,
    duration: &Option<Duration>,
    refresh: &Duration,
    spinner: char,
) -> Result<String> {
    let duration = duration.unwrap_or_default();
    let duration_millis = duration.num_milliseconds();
    if duration_millis == 0 || refresh.num_milliseconds() == 0 {
        return Ok(ofmt!(timestamp, "running [{}]", spinner));
    }
    let head = ofmt!(timestamp, "running ");
    let tail = format!(" [{}]", spinner);
    let barsize = {
        let b = (duration_millis / refresh.num_milliseconds()) as usize;
        let overhead = head.len() + tail.len() + 1;
        debug_assert!(
            width >= overhead,
            "width {} not greater than overhead {}",
            width,
            overhead
        );
        if b + overhead > width {
            width - overhead
        } else {
            b
        }
    };
    let msg = if barsize <= 1 {
        ofmt!(timestamp, "running [{}]", spinner)
    } else {
        let elapsed = now - start;
        let ratio = elapsed.num_milliseconds() as f32 / duration_millis as f32;
        let left = if ratio < 1_f32 {
            ((barsize as f32) * ratio).ceil() as usize
        } else {
            barsize
        };
        let right = barsize.saturating_sub(left);
        let marker = if elapsed > duration { "=" } else { ">" };
        format!(
            "{}[{:=>left$}{:right$}]{}",
            head,
            marker,
            "",
            tail,
            left = left,
            right = right
        )
    };
    Ok(msg)
}
