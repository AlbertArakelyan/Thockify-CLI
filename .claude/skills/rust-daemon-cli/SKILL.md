---
name: rust-daemon-cli
description: Guide for building daemon-style CLI tools in Rust on Windows. Use when implementing background processes, PID management, start/stop commands, or detached process spawning.
user-invocable: false
---

# Daemon-Style CLI Architecture in Rust (Windows)

## Problem

You need a CLI tool where the user runs a command (`start`), the tool launches a long-running process in the background, and the terminal returns immediately. The user can later run `stop` to kill it. No external service managers, no third-party daemon crates — pure Rust with standard library and clap.

## Architecture

The trick is a **hidden subcommand**. The CLI has a public `start` command and a hidden `run` command. `start` spawns a detached child process that executes `run`, writes the child's PID to a file, and exits. `stop` reads the PID file and kills the process.

```
thok start   →  spawns "thok run" detached  →  exits
thok stop    →  reads PID, kills process     →  exits
thok run     →  (hidden) actually does work  →  runs until killed
```

## Clap Structure

Use `Option<Commands>` for the subcommand field so that the binary can handle global flags (like `--profile`) without a subcommand.

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "myapp")]
struct Cli {
    #[arg(long)]
    some_flag: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Start,
    Stop,
    #[command(hide = true)]  // hidden from --help
    Run,
}
```

This allows `myapp --some-flag value` (no subcommand, just saves config and exits) and `myapp start` to coexist naturally. `myapp --some-flag value start` also works — flag is processed first, then subcommand.

## Spawning a Detached Process on Windows

Use `std::os::windows::process::CommandExt` for the `creation_flags()` method. The flag `CREATE_NO_WINDOW` (0x08000000) prevents a console window from appearing.

```rust
use std::process::{Command, Stdio};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

fn start() {
    let exe = std::env::current_exe().expect("Failed to get exe path");

    #[cfg(windows)]
    let child = Command::new(&exe)
        .arg("run")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn();

    #[cfg(not(windows))]
    let child = Command::new(&exe)
        .arg("run")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    match child {
        Ok(child) => {
            write_pid(child.id());
            println!("Started (PID {})", child.id());
        }
        Err(e) => {
            eprintln!("Failed to start: {e}");
            std::process::exit(1);
        }
    }
}
```

### Key details

- **`Stdio::null()` on stdout/stderr** — Without this, the child's output leaks into the parent's terminal before it exits. Always null both streams for a clean detached process.
- **`CREATE_NO_WINDOW`** — Without this flag on Windows, a console window flashes or stays open for the child process.
- **`std::env::current_exe()`** — Spawns the same binary. Works with both `cargo run` and installed binaries.
- **No external crates needed** — Everything comes from `std::process` and `std::os::windows`.

## PID File Management

Store the PID as plain text in a known config directory. Use the `dirs` crate for cross-platform paths.

```rust
fn config_dir() -> PathBuf {
    dirs::config_dir()
        .expect("Could not determine config directory")
        .join("myapp")
}

fn pid_path() -> PathBuf {
    config_dir().join("myapp.pid")
}

fn write_pid(pid: u32) {
    let dir = config_dir();
    std::fs::create_dir_all(&dir).expect("Failed to create config dir");
    std::fs::write(pid_path(), pid.to_string()).expect("Failed to write PID");
}

fn read_pid() -> Option<u32> {
    std::fs::read_to_string(pid_path()).ok()?.trim().parse().ok()
}
```

## Stopping the Process (Windows)

Use `taskkill` for killing and `tasklist` for checking if a process is alive.

```rust
#[cfg(windows)]
fn is_process_alive(pid: u32) -> bool {
    Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/NH"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()))
        .unwrap_or(false)
}

#[cfg(windows)]
fn kill_process(pid: u32) -> bool {
    Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/F"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
```

For Unix, use `kill -0` (alive check) and `kill` (terminate).

## Edge Cases to Handle

| Scenario | Solution |
|---|---|
| Stale PID file (process crashed) | Check `is_process_alive()` before starting. Remove stale file and proceed. |
| Double start | Detect running process, tell user to stop first. |
| Stop when not running | Print "not running", exit 0. Not an error. |
| PID file contains garbage | `read_pid()` returns `None` on parse failure, treat as not running. |
| Config directory doesn't exist | `create_dir_all()` before writing PID or settings. |

## Settings Persistence

Use `serde` + `serde_json` with a simple struct for settings that persist between commands.

```rust
#[derive(Serialize, Deserialize, Default)]
struct Settings {
    #[serde(skip_serializing_if = "Option::is_none")]
    profile: Option<String>,
}
```

Store at `config_dir().join("settings.json")`. Load with fallback to defaults on missing/corrupt file:

```rust
fn load_settings() -> Settings {
    std::fs::read_to_string(settings_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}
```

## When to Apply

Any Rust CLI tool that needs a long-running background process with start/stop semantics — sound engines, file watchers, local servers, monitoring agents. This approach avoids the complexity of platform-specific service managers while still giving clean daemon behavior.
