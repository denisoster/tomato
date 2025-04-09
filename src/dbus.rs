use super::timer::{PomodoroTimer, TimerMode, TimerState};
use smol::Timer;
use std::sync::Mutex;
use std::time::Duration;
use zbus::{connection, interface};

pub struct PomodoroServer {
    pub timer: Mutex<PomodoroTimer>,
}
#[interface(name = "org.pomodoro.Timer")]
impl PomodoroServer {
    async fn start(&self) {
        self.timer.lock().unwrap().start();
    }

    async fn pause(&self) {
        self.timer.lock().unwrap().pause();
    }

    async fn stop(&self) {
        self.timer.lock().unwrap().stop();
    }

    async fn restart(&self) {
        self.timer.lock().unwrap().restart();
    }

    async fn skip(&self) {
        self.timer.lock().unwrap().skip();
    }

    async fn reset_counter(&self) {
        self.timer.lock().unwrap().reset_counter();
    }

    async fn get_status(&self) -> String {
        let mut timer = self.timer.lock().unwrap();
        timer.update();
        timer.get_status()
    }

    #[zbus(property)]
    async fn mode(&self) -> String {
        match self.timer.lock().unwrap().get_mode() {
            TimerMode::Work => "Work".to_string(),
            TimerMode::ShortBreak => "ShortBreak".to_string(),
            TimerMode::LongBreak => "LongBreak".to_string(),
        }
    }

    #[zbus(property)]
    async fn state(&self) -> String {
        match self.timer.lock().unwrap().get_state() {
            TimerState::Stopped => "Stopped".to_string(),
            TimerState::Running => "Running".to_string(),
            TimerState::Paused => "Paused".to_string(),
        }
    }
    #[zbus(property)]
    async fn remaining(&self) -> u64 {
        self.timer.lock().unwrap().get_remaining().as_secs()
    }

    #[zbus(property)]
    async fn current_session(&self) -> u32 {
        self.timer.lock().unwrap().get_current_session()
    }
}

pub async fn server() -> zbus::Result<()> {
    let server = PomodoroServer {
        timer: Mutex::new(PomodoroTimer::default()),
    };

    let _connection = connection::Builder::session()?
        .name("org.pomodoro.Timer")?
        .serve_at("/org/pomodoro/Timer", server)?
        .build()
        .await?;

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}
