// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use tokio::time;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    Running,
    Sleeping,
}

pub struct Progbar {
    pb: indicatif::ProgressBar,
    start: time::Instant,
    hidden: bool,
    duration: time::Duration,
    refresh_delay: time::Duration,
    mode: Mode,
    mode_wanted: Mode,
}

impl Default for Progbar {
    fn default() -> Progbar {
        Progbar {
            pb: indicatif::ProgressBar::hidden(),
            start: time::Instant::now(),
            hidden: true,
            duration: time::Duration::from_secs(0),
            refresh_delay: time::Duration::from_millis(250),
            mode: Mode::Running,
            mode_wanted: Mode::Running,
        }
    }
}

impl Progbar {
    pub fn set_running(&mut self, duration: time::Duration) {
        self.mode_wanted = Mode::Running;
        self.duration = duration;
        self.start = time::Instant::now();
    }

    pub fn set_sleep(&mut self, duration: time::Duration) {
        self.mode_wanted = Mode::Sleeping;
        self.duration = duration;
        self.start = time::Instant::now();
    }

    pub fn do_switch_mode(&mut self) {
        if self.mode == self.mode_wanted {
            return;
        }
        self.hide();
        self.mode = self.mode_wanted;
        self.show();
    }

    fn bar_size(overhead: usize, dur: u128, refresh: u128) -> usize {
        let max_width = if let Some((w, _)) = term_size::dimensions() {
            w
        } else {
            80
        };
        let bar_size = (dur / refresh) as usize;
        if bar_size + overhead > max_width {
            max_width - overhead
        } else {
            bar_size
        }
    }

    fn create_indicatif_pb(
        mode: Mode,
        duration: time::Duration,
        refresh_delay: time::Duration,
    ) -> indicatif::ProgressBar {
        let pb = indicatif::ProgressBar::hidden();
        let dur = duration.as_millis();
        let refresh = refresh_delay.as_millis();
        let fmt = match mode {
            Mode::Sleeping => {
                let header = "=> sleeping";
                let bar_size = Progbar::bar_size(header.len() + 1, dur, refresh);
                format!("{}{{bar:{}}}", header, bar_size)
            }
            Mode::Running => {
                if dur <= 3000 {
                    String::from("=> running [{spinner}]")
                } else {
                    let header = "=> running ";
                    let bar_size = Progbar::bar_size(header.len() + 5, dur, refresh);
                    format!("{}[{{bar:{}}}] {{spinner}}", header, bar_size)
                }
            }
        };
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template(&fmt)
                .progress_chars(if mode == Mode::Sleeping { ".. " } else { "=>-" })
                .tick_chars("-\\|/ "),
        );
        pb.set_length(Progbar::pos_from_dur(duration));
        pb
    }

    fn pos_from_dur(duration: time::Duration) -> u64 {
        duration.as_millis() as u64
    }

    fn elapsed(&self) -> time::Duration {
        time::Instant::now() - self.start
    }

    pub fn hide(&mut self) {
        self.pb.finish_and_clear();
        self.pb = Progbar::create_indicatif_pb(self.mode, self.duration, self.refresh_delay);
        self.hidden = true;
    }

    pub fn show(&mut self) {
        self.pb = Progbar::create_indicatif_pb(self.mode, self.duration, self.refresh_delay);
        self.pb
            .set_draw_target(indicatif::ProgressDrawTarget::stderr());
        self.hidden = false;
        self.refresh();
    }

    pub fn refresh(&mut self) {
        if self.mode != self.mode_wanted {
            self.do_switch_mode();
        }
        self.pb.set_position(Progbar::pos_from_dur(self.elapsed()));
    }
}
