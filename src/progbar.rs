// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use tokio::time;

pub struct Progbar {
    pb: indicatif::ProgressBar,
    start: time::Instant,
    hidden: bool,
    duration: time::Duration,
    msg: String,
    sleep: bool,
}

impl Progbar {
    pub fn new_timer(msg: &str, duration: time::Duration) -> Progbar {
        Progbar::new_with_sleep(msg, duration, false)
    }

    pub fn new_sleep(msg: &str, duration: time::Duration) -> Progbar {
        Progbar::new_with_sleep(msg, duration, true)
    }

    pub fn new_with_sleep(msg: &str, duration: time::Duration, sleep: bool) -> Progbar {
        Progbar {
            pb: Progbar::create_indicatif_pb(msg, duration, sleep),
            start: time::Instant::now(),
            hidden: true,
            duration,
            msg: msg.to_string(),
            sleep,
        }
    }

    fn create_indicatif_pb(
        msg: &str,
        duration: time::Duration,
        sleep: bool,
    ) -> indicatif::ProgressBar {
        let pb = indicatif::ProgressBar::hidden();
        let onlyspin = duration.as_millis() == 0;
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template(if onlyspin {
                    "{msg:12} [{spinner}]"
                } else {
                    "{msg:12} [{bar:55}] {spinner}"
                })
                .progress_chars(if sleep { "-<=" } else { "=>-" })
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

    pub fn set_message(&mut self, msg: &str) {
        self.msg = msg.to_string();
        self.pb.set_message(msg);
    }

    pub fn hide(&mut self) {
        self.pb.finish_and_clear();
        self.pb = Progbar::create_indicatif_pb(&self.msg, self.duration, self.sleep);
        self.hidden = true;
    }

    pub fn refresh(&mut self) {
        if self.hidden {
            self.pb
                .set_draw_target(indicatif::ProgressDrawTarget::stderr());
            self.hidden = false;
        }
        self.pb.set_position(Progbar::pos_from_dur(self.elapsed()));
        self.hidden = false;
    }
}
