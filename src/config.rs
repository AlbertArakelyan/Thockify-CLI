use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const PACK_DIR: &str = "sound-packs";

#[derive(Serialize, Deserialize, Default)]
pub struct Settings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
}

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .expect("Could not determine config directory")
        .join("thockify")
}

pub fn settings_path() -> PathBuf {
    config_dir().join("settings.json")
}

pub fn pid_path() -> PathBuf {
    config_dir().join("thok.pid")
}

pub fn sound_packs_dir() -> PathBuf {
    // Try relative to exe first (for installed binary), fall back to CWD
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let candidate = exe_dir.join(PACK_DIR);
            if candidate.is_dir() {
                return candidate;
            }
        }
    }
    PathBuf::from(PACK_DIR)
}

pub fn load_settings() -> Settings {
    let path = settings_path();
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Settings::default(),
    }
}

pub fn set_profile(name: &str) {
    let dir = config_dir();
    std::fs::create_dir_all(&dir).expect("Failed to create config directory");

    let mut settings = load_settings();
    settings.profile = Some(name.to_string());

    let json = serde_json::to_string_pretty(&settings).expect("Failed to serialize settings");
    std::fs::write(settings_path(), json).expect("Failed to write settings.json");
}

pub fn list_profiles() {
    let dir = sound_packs_dir();
    match std::fs::read_dir(&dir) {
        Ok(entries) => {
            let mut found = false;
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    println!("  {}", entry.file_name().to_string_lossy());
                    found = true;
                }
            }
            if !found {
                println!("No sound packs found in {}", dir.display());
            }
        }
        Err(_) => {
            eprintln!("Sound packs directory not found: {}", dir.display());
            std::process::exit(1);
        }
    }
}
