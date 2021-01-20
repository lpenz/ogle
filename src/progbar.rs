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

impl Default for Progbar {
    fn default() -> Progbar {
        Progbar {
            pb: indicatif::ProgressBar::hidden(),
            start: time::Instant::now(),
            hidden: true,
            duration: time::Duration::from_secs(0),
            msg: "".to_string(),
            sleep: false,
        }
    }
}

impl Progbar {
    pub fn set_timer(&mut self, msg: &str, duration: time::Duration) {
        self.hide();
        self.duration = duration;
        self.msg = msg.to_string();
        self.sleep = false;
        self.show();
    }

    pub fn set_sleep(&mut self, msg: &str, duration: time::Duration) {
        self.hide();
        self.duration = duration;
        self.msg = msg.to_string();
        self.sleep = true;
        self.show();
    }

    fn create_indicatif_pb(
        msg: &str,
        duration: time::Duration,
        sleep: bool,
    ) -> indicatif::ProgressBar {
        let pb = indicatif::ProgressBar::hidden();
        let dur = duration.as_millis();
        let fmt = format!("{{msg}}{{bar:{}}}", dur / 300);
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template(if sleep {
                    fmt.as_str()
                } else if dur == 0 {
                    "{msg:12} [{spinner}]"
                } else if dur <= 1000 {
                    ""
                } else if dur <= 3000 {
                    "{msg:12} [{spinner}]"
                } else {
                    "{msg:12} [{bar:55}] {spinner}"
                })
                .progress_chars(if sleep { ".. " } else { "=>-" })
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

    pub fn show(&mut self) {
        self.pb = Progbar::create_indicatif_pb(&self.msg, self.duration, self.sleep);
        self.pb
            .set_draw_target(indicatif::ProgressDrawTarget::stderr());
        self.hidden = false;
        self.refresh();
    }

    pub fn refresh(&mut self) {
        self.pb.set_position(Progbar::pos_from_dur(self.elapsed()));
    }
}
