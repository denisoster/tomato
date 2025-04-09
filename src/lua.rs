use mlua::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicBool, Ordering};

struct TimerState {
    running: Arc<AtomicBool>,
    duration_secs: Arc<Mutex<u64>>,
    initial_duration: u64,
}

#[derive(Clone)]
struct LuaTimer {
    state: Arc<TimerState>,
}

impl LuaTimer {
    fn new(_: &Lua, seconds: u64) -> LuaResult<Self> {
        Ok(Self {
            state: Arc::new(TimerState {
                running: Arc::new(AtomicBool::new(false)),
                duration_secs: Arc::new(Mutex::new(seconds)),
                initial_duration: seconds,
            })
        })
    }
}

impl LuaUserData for LuaTimer {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("start", |_, this, ()| {
            if !this.state.running.swap(true, Ordering::Relaxed) {
                let state = this.state.clone();
                let initial_duration = state.initial_duration;
                *state.duration_secs.lock().unwrap() = initial_duration;

                thread::spawn(move || {
                    let start_time = Instant::now();
                    let mut last_remaining = initial_duration;

                    while state.running.load(Ordering::Relaxed) {
                        let elapsed = start_time.elapsed().as_secs();
                        let remaining = initial_duration.saturating_sub(elapsed);

                        if remaining != last_remaining {
                            *state.duration_secs.lock().unwrap() = remaining;
                            last_remaining = remaining;
                        }

                        if remaining == 0 {
                            state.running.store(false, Ordering::Relaxed);
                            break;
                        }

                        thread::sleep(Duration::from_millis(100));
                    }
                });
            }
            Ok(())
        });

        methods.add_method("stop", |_, this, ()| {
            this.state.running.store(false, Ordering::Relaxed);
            Ok(())
        });

        methods.add_method("reset", |_, this, ()| {
            this.state.running.store(false, Ordering::Relaxed);
            *this.state.duration_secs.lock().unwrap() = this.state.initial_duration;
            Ok(())
        });

        methods.add_method("get_remaining", |_, this, ()| {
            Ok(*this.state.duration_secs.lock().unwrap())
        });
    }
}

#[mlua::lua_module]
fn libtomato(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("new", lua.create_function(|lua, seconds: u64| {
        LuaTimer::new(lua, seconds)
    })?)?;

    Ok(exports)
}