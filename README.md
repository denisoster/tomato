# Tomato (トマト)

Interactive mode
```
Pomodoro Timer
Commands:
s - Start/Resume
p - Pause
t - Stop
r - Restart current session
k - Skip to next session
c - Reset session counter
q - Quit
Mode: Work | State: Stopped | Remaining: 25:00 | Session: 0/4>
```

Daemon mode

```
[denis@black tomato]$ target/debug/tomato --daemon &
```

Client 
```
[denis@black ~]$ busctl --user call org.pomodoro.Timer /org/pomodoro/Timer org.pomodoro.Timer GetStatus
s "Mode: Work | State: Running | Remaining: 24:55 | Session: 0/4"
[denis@black ~]$ busctl --user get-property org.pomodoro.Timer /org/pomodoro/Timer org.pomodoro.Timer Remaining
t 1495
[denis@black ~]$ busctl --user get-property org.pomodoro.Timer /org/pomodoro/Timer org.pomodoro.Timer State
s "Running"
[denis@black ~]$ busctl --user get-property org.pomodoro.Timer /org/pomodoro/Timer org.pomodoro.Timer Mode
s "Work"
[denis@black ~]$ busctl --user call org.pomodoro.Timer /org/pomodoro/Timer org.pomodoro.Timer Skip
[denis@black ~]$ busctl --user get-property org.pomodoro.Timer /org/pomodoro/Timer org.pomodoro.Timer Mode
s "ShortBreak"
```

Lua
```
Starting timer...
Remaining: 4 (Δ 0.996 s)
Remaining: 3 (Δ 1.993 s)
Remaining: 2 (Δ 2.989 s)
Remaining: 1 (Δ 3.985 s)

Timer finished! (CPU: 4.981 s, Wall: 5 s)
```