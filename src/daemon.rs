use std::process::{Command, Stdio};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn start() {
    let settings = crate::config::load_settings();
    let profile = settings.profile.unwrap_or_else(|| {
        eprintln!("No profile set. Run: thok --profile <name>");
        std::process::exit(1);
    });

    let pack_dir = crate::config::sound_packs_dir().join(&profile);
    if !pack_dir.is_dir() {
        eprintln!("Sound pack not found: {}", pack_dir.display());
        std::process::exit(1);
    }

    // Check if already running
    if let Some(pid) = read_pid() {
        if is_process_alive(pid) {
            println!("Already running (PID {pid}). Use `thok stop` first.");
            return;
        }
        // Stale PID file, clean up
        let _ = std::fs::remove_file(crate::config::pid_path());
    }

    let exe = std::env::current_exe().expect("Failed to get current executable path");

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
            let pid = child.id();
            let dir = crate::config::config_dir();
            std::fs::create_dir_all(&dir).expect("Failed to create config directory");
            std::fs::write(crate::config::pid_path(), pid.to_string())
                .expect("Failed to write PID file");
            println!("Thockify started (PID {pid}, profile: {profile})");
        }
        Err(e) => {
            eprintln!("Failed to start: {e}");
            std::process::exit(1);
        }
    }
}

pub fn stop() {
    let pid_path = crate::config::pid_path();
    let Some(pid) = read_pid() else {
        println!("Not running.");
        return;
    };

    if kill_process(pid) {
        let _ = std::fs::remove_file(&pid_path);
        println!("Thockify stopped (PID {pid}).");
    } else {
        // Process might already be dead
        let _ = std::fs::remove_file(&pid_path);
        println!("Process {pid} not found (cleaned up PID file).");
    }
}

fn read_pid() -> Option<u32> {
    let content = std::fs::read_to_string(crate::config::pid_path()).ok()?;
    content.trim().parse().ok()
}

#[cfg(windows)]
fn is_process_alive(pid: u32) -> bool {
    Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/NH"])
        .output()
        .map(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.contains(&pid.to_string())
        })
        .unwrap_or(false)
}

#[cfg(not(windows))]
fn is_process_alive(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(windows)]
fn kill_process(pid: u32) -> bool {
    Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/F"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(not(windows))]
fn kill_process(pid: u32) -> bool {
    Command::new("kill")
        .arg(pid.to_string())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
