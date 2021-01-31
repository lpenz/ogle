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
    msg: String,
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
            msg: "".to_string(),
            mode: Mode::Running,
            mode_wanted: Mode::Running,
        }
    }
}

impl Progbar {
    pub fn set_running(&mut self, duration: time::Duration) {
        self.mode_wanted = Mode::Running;
        self.duration = duration;
        self.msg = "=> running".to_string();
        self.start = time::Instant::now();
    }

    pub fn set_sleep(&mut self, duration: time::Duration) {
        self.mode_wanted = Mode::Sleeping;
        self.duration = duration;
        self.msg = "=> sleeping".to_string();
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

    fn create_indicatif_pb(
        msg: &str,
        mode: Mode,
        duration: time::Duration,
    ) -> indicatif::ProgressBar {
        let pb = indicatif::ProgressBar::hidden();
        let dur = duration.as_millis();
        let fmt = format!("{{msg}}{{bar:{}}}", dur / 300);
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template(if mode == Mode::Sleeping {
                    fmt.as_str()
                } else if dur <= 3000 {
                    "{msg:12} [{spinner}]"
                } else {
                    "{msg:12} [{bar:55}] {spinner}"
                })
                .progress_chars(if mode == Mode::Sleeping { ".. " } else { "=>-" })
                .tick_chars("-\\|/ "),
        );
        pb.set_message(msg);
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
        self.pb = Progbar::create_indicatif_pb(&self.msg, self.mode, self.duration);
        self.hidden = true;
    }

    pub fn show(&mut self) {
        self.pb = Progbar::create_indicatif_pb(&self.msg, self.mode, self.duration);
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
