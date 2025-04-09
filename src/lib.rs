use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimerState {
    Stopped,
    Running,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimerMode {
    Work,
    ShortBreak,
    LongBreak,
}

#[derive(Debug)]
pub struct PomodoroTimer {
    work_duration: Duration,
    short_break_duration: Duration,
    long_break_duration: Duration,
    sessions_before_long_break: u32,
    current_session: u32,
    mode: TimerMode,
    state: TimerState,
    remaining: Duration,
    start_time: Option<Instant>,
    pause_time: Option<Instant>,
    last_update: Option<Instant>,
}

#[derive(Debug)]
pub enum TimerCommand {
    Start,
    Pause,
    Stop,
    Restart,
    Skip,
    ResetCounter,
}

impl PomodoroTimer {
    pub fn new(
        work_duration: Duration,
        short_break_duration: Duration,
        long_break_duration: Duration,
        sessions_before_long_break: u32,
    ) -> Self {
        PomodoroTimer {
            work_duration,
            short_break_duration,
            long_break_duration,
            sessions_before_long_break,
            current_session: 0,
            mode: TimerMode::Work,
            state: TimerState::Stopped,
            remaining: work_duration,
            start_time: None,
            pause_time: None,
            last_update: None,
        }
    }

    pub fn default() -> Self {
        Self::new(
            Duration::from_secs(25 * 60),
            Duration::from_secs(5 * 60),
            Duration::from_secs(15 * 60),
            4,
        )
    }

    pub fn start(&mut self) {
        match self.state {
            TimerState::Stopped => {
                self.start_time = Some(Instant::now());
                self.last_update = self.start_time;
                self.state = TimerState::Running;
            }
            TimerState::Paused => {
                if let Some(pause_time) = self.pause_time {
                    let paused_duration = pause_time.elapsed();
                    if let Some(start_time) = &mut self.start_time {
                        *start_time += paused_duration;
                    }
                    self.pause_time = None;
                    self.last_update = Some(Instant::now());
                    self.state = TimerState::Running;
                }
            }
            TimerState::Running => {}
        }
    }

    pub fn pause(&mut self) {
        if self.state == TimerState::Running {
            self.pause_time = Some(Instant::now());
            self.state = TimerState::Paused;
        }
    }

    pub fn stop(&mut self) {
        self.state = TimerState::Stopped;
        self.start_time = None;
        self.pause_time = None;
        self.last_update = None;
    }

    pub fn restart(&mut self) {
        self.stop();
        match self.mode {
            TimerMode::Work => self.remaining = self.work_duration,
            TimerMode::ShortBreak => self.remaining = self.short_break_duration,
            TimerMode::LongBreak => self.remaining = self.long_break_duration,
        }
        self.start();
    }

    pub fn skip(&mut self) {
        self.stop();
        self.next_mode();
        self.start();
    }

    pub fn reset_counter(&mut self) {
        self.current_session = 0;
    }

    fn next_mode(&mut self) {
        match self.mode {
            TimerMode::Work => {
                self.current_session += 1;
                if self.current_session % self.sessions_before_long_break == 0 {
                    self.mode = TimerMode::LongBreak;
                    self.remaining = self.long_break_duration;
                } else {
                    self.mode = TimerMode::ShortBreak;
                    self.remaining = self.short_break_duration;
                }
            }
            TimerMode::ShortBreak | TimerMode::LongBreak => {
                self.mode = TimerMode::Work;
                self.remaining = self.work_duration;
            }
        }
    }

    pub fn update(&mut self) -> Option<Duration> {
        if let TimerState::Running = self.state {
            if let (Some(_start_time), Some(last_update)) = (self.start_time, self.last_update) {
                let now = Instant::now();
                let elapsed_since_last_update = now - last_update;
                self.last_update = Some(now);

                if elapsed_since_last_update >= self.remaining {
                    self.stop();
                    self.next_mode();
                    return Some(Duration::from_secs(0));
                } else {
                    self.remaining -= elapsed_since_last_update;
                    return Some(self.remaining);
                }
            }
        }
        None
    }

    pub fn execute_command(&mut self, cmd: TimerCommand) {
        match cmd {
            TimerCommand::Start => self.start(),
            TimerCommand::Pause => self.pause(),
            TimerCommand::Stop => self.stop(),
            TimerCommand::Restart => self.restart(),
            TimerCommand::Skip => self.skip(),
            TimerCommand::ResetCounter => self.reset_counter(),
        }
    }

    pub fn get_status(&self) -> String {
        let mode_str = match self.mode {
            TimerMode::Work => "Work",
            TimerMode::ShortBreak => "Short Break",
            TimerMode::LongBreak => "Long Break",
        };

        let state_str = match self.state {
            TimerState::Stopped => "Stopped",
            TimerState::Running => "Running",
            TimerState::Paused => "Paused",
        };

        let remaining = {
            let secs = self.remaining.as_secs();
            format!("{:02}:{:02}", secs / 60, secs % 60)
        };

        format!(
            "Mode: {} | State: {} | Remaining: {} | Session: {}/{}",
            mode_str,
            state_str,
            remaining,
            self.current_session,
            self.sessions_before_long_break
        )
    }

    pub fn get_mode(&self) -> TimerMode {
        self.mode
    }

    pub fn get_state(&self) -> TimerState {
        self.state
    }

    pub fn get_remaining(&self) -> Duration {
        self.remaining
    }

    pub fn get_current_session(&self) -> u32 {
        self.current_session
    }
}
