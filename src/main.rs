use std::time::{Duration};
use std::{env, thread};
use std::io::{self, Write};
use std::sync::mpsc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tomato::{PomodoroTimer, TimerCommand};
use tomato::dbus::server;

fn main() {
    let args: Vec<String> = env::args().collect();
    let daemon_mode = args.len() > 1 && args[1] == "--daemon";

    if daemon_mode {
        if let Err(e) = run_daemon() {
            eprintln!("Error running daemon: {}", e);
            std::process::exit(1);
        }
    } else {
        run_interactive()
    }
}

fn run_interactive() {
    let mut timer = PomodoroTimer::default();
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
                "s" => tx.send(TimerCommand::Start).unwrap(),
                "p" => tx.send(TimerCommand::Pause).unwrap(),
                "t" => tx.send(TimerCommand::Stop).unwrap(),
                "r" => tx.send(TimerCommand::Restart).unwrap(),
                "k" => tx.send(TimerCommand::Skip).unwrap(),
                "c" => tx.send(TimerCommand::ResetCounter).unwrap(),
                "q" => {
                    running_clone.store(false, Ordering::Relaxed);
                    break;
                }
                _ => println!("Unknown command"),
            }
        }
    });

    while running.load(Ordering::Relaxed) {
        if let Ok(cmd) = rx.try_recv() {
            timer.execute_command(cmd);
        }

        timer.update();

        print!("\r{}", timer.get_status());
        io::stdout().flush().unwrap();

        thread::sleep(Duration::from_millis(100));
    }

    println!("\nPomodoro Timer stopped.");
}

fn run_daemon() -> zbus::Result<()> {
    smol::block_on(server())
}