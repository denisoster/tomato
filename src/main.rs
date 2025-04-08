use std::time::{Duration, Instant};
use std::thread;
use std::io::{self, Write};
use std::sync::mpsc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

enum Command {
    Start,
    Pause,
    Stop,
    Restart,
    Skip,
    ResetCounter,
    Quit,
}

enum TimerState {
    Stopped,
    Running,
    Paused,
}

enum TimerMode {
    Work,
    ShortBreak,
    LongBreak,
}

struct PomodoroTimer {
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

impl PomodoroTimer {
    fn new() -> Self {
        PomodoroTimer {
            work_duration: Duration::from_secs(25 * 60),
            short_break_duration: Duration::from_secs(5 * 60),
            long_break_duration: Duration::from_secs(15 * 60),
            sessions_before_long_break: 4,
            current_session: 0,
            mode: TimerMode::Work,
            state: TimerState::Stopped,
            remaining: Duration::from_secs(25 * 60),
            start_time: None,
            pause_time: None,
            last_update: None,
        }
    }

    fn start(&mut self) {
        if let TimerState::Stopped = self.state {
            self.start_time = Some(Instant::now());
            self.last_update = self.start_time;
            self.state = TimerState::Running;
            println!("Timer started!");
        } else if let TimerState::Paused = self.state {
            if let Some(pause_time) = self.pause_time {
                let paused_duration = pause_time.elapsed();
                if let Some(start_time) = &mut self.start_time {
                    *start_time += paused_duration;
                }
                self.pause_time = None;
                self.last_update = Some(Instant::now());
                self.state = TimerState::Running;
                println!("Timer resumed!");
            }
        }
    }

    fn pause(&mut self) {
        if let TimerState::Running = self.state {
            self.pause_time = Some(Instant::now());
            self.state = TimerState::Paused;
            println!("Timer paused!");
        }
    }

    fn stop(&mut self) {
        self.state = TimerState::Stopped;
        self.start_time = None;
        self.pause_time = None;
        self.last_update = None;
        println!("Timer stopped!");
    }

    fn restart(&mut self) {
        self.stop();
        match self.mode {
            TimerMode::Work => self.remaining = self.work_duration,
            TimerMode::ShortBreak => self.remaining = self.short_break_duration,
            TimerMode::LongBreak => self.remaining = self.long_break_duration,
        }
        self.start();
    }

    fn skip(&mut self) {
        self.stop();
        self.next_mode();
        self.start();
    }

    fn reset_counter(&mut self) {
        self.current_session = 0;
        println!("Session counter reset!");
    }

    fn next_mode(&mut self) {
        match self.mode {
            TimerMode::Work => {
                self.current_session += 1;
                if self.current_session % self.sessions_before_long_break == 0 {
                    self.mode = TimerMode::LongBreak;
                    self.remaining = self.long_break_duration;
                    println!("Long break started!");
                } else {
                    self.mode = TimerMode::ShortBreak;
                    self.remaining = self.short_break_duration;
                    println!("Short break started!");
                }
            }
            TimerMode::ShortBreak | TimerMode::LongBreak => {
                self.mode = TimerMode::Work;
                self.remaining = self.work_duration;
                println!("Work session started!");
            }
        }
    }

    fn update(&mut self) -> Option<Duration> {
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

    fn get_status(&mut self) -> String {
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
}

fn main() {
    let mut timer = PomodoroTimer::new();
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    println!("Pomodoro Timer");
    println!("Commands:");
    println!("  s - Start/Resume");
    println!("  p - Pause");
    println!("  t - Stop");
    println!("  r - Restart current session");
    println!("  k - Skip to next session");
    println!("  c - Reset session counter");
    println!("  q - Quit");

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        loop {
            print!("> ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();

            match input.trim() {
                "s" => tx.send(Command::Start).unwrap(),
                "p" => tx.send(Command::Pause).unwrap(),
                "t" => tx.send(Command::Stop).unwrap(),
                "r" => tx.send(Command::Restart).unwrap(),
                "k" => tx.send(Command::Skip).unwrap(),
                "c" => tx.send(Command::ResetCounter).unwrap(),
                "q" => {
                    tx.send(Command::Quit).unwrap();
                    break;
                }
                _ => println!("Unknown command"),
            }
        }
    });

    while running_clone.load(Ordering::Relaxed) {
        if let Ok(cmd) = rx.try_recv() {
            match cmd {
                Command::Start => timer.start(),
                Command::Pause => timer.pause(),
                Command::Stop => timer.stop(),
                Command::Restart => timer.restart(),
                Command::Skip => timer.skip(),
                Command::ResetCounter => timer.reset_counter(),
                Command::Quit => {
                    running_clone.store(false, Ordering::Relaxed);
                    break;
                }
            }
        }

        timer.update();

        print!("\r{}", timer.get_status());
        io::stdout().flush().unwrap();

        thread::sleep(Duration::from_millis(100));
    }

    println!("\nPomodoro Timer stopped.");
}