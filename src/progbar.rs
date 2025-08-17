// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use crate::time_wrapper::Duration;
use crate::time_wrapper::Instant;
use color_eyre::Result;

// Basic functions:

fn get_prefix(unchanged_runs: u32) -> String {
    if unchanged_runs == 0 {
        "".to_string()
    } else {
        format!("unchanged {unchanged_runs}, ")
    }
}

pub fn progbar_sleeping(
    unchanged_runs: u32,
    sleep: &Duration,
    now: &Instant,
    deadline: &Instant,
    spinner: char,
) -> String {
    let prefix = get_prefix(unchanged_runs);
    if sleep.num_seconds() > 1 {
        let left = deadline - now;
        let left = left.num_seconds() + 1;
        format!("{prefix}sleeping for {left}s [{spinner}]")
    } else {
        format!("{prefix}sleeping [{spinner}]")
    }
}

pub fn progbar_running(
    width: usize,
    unchanged_runs: u32,
    now: &Instant,
    start: &Instant,
    duration: Option<Duration>,
    refresh: &Duration,
    spinner: char,
) -> Result<String> {
    let duration = duration.unwrap_or_default();
    let duration_millis = duration.num_milliseconds();
    let prefix = get_prefix(unchanged_runs);
    if duration_millis == 0 || refresh.num_milliseconds() == 0 {
        return Ok(format!("{prefix}running [{spinner}]"));
    }
    let head = format!("{prefix}running [");
    let tail = format!("] [{spinner}]");
    let barsize = {
        let b = usize::try_from(duration_millis / refresh.num_milliseconds())?;
        let overhead = head.len() + tail.len() + 1;
        debug_assert!(
            width >= overhead,
            "width {width} not greater than overhead {overhead}",
        );
        if b + overhead > width {
            width - overhead
        } else {
            b
        }
    };
    Ok(if barsize <= 1 {
        format!("{prefix}running [{spinner}]")
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
            "{}{:=>left$}{:right$}{}",
            head,
            marker,
            "",
            tail,
            left = left,
            right = right
        )
    })
}

pub fn spinner_get(spinner: &mut char) -> char {
    *spinner = match spinner {
        '/' => '-',
        '-' => '\\',
        '\\' => '|',
        '|' => '/',
        _ => {
            panic!("unknown spinner position [{spinner}]")
        }
    };
    *spinner
}

// Tests /////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn spinner() {
        let spins = (1..6)
            .into_iter()
            .scan('/', |spinner, _| Some(spinner_get(spinner)))
            .collect::<Vec<_>>();
        assert_eq!(spins, vec!['-', '\\', '|', '/', '-']);
    }

    #[should_panic]
    #[test]
    fn spinner_panic() {
        let mut spinner = '0';
        let _ = spinner_get(&mut spinner);
    }
}
