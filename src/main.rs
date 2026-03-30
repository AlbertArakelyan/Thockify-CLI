use rdev::{listen, Event, EventType, Key};
use rodio::{Decoder, OutputStream, Source};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

const PACK_DIR: &str = "sound-packs/topre";

fn main() {
    println!("Starting MechVibes CLI...");
    println!("Sound pack: topre");
    println!("Press any key anywhere to hear sounds!");
    println!("Press Ctrl+C to exit");

    // Pre-load all sound files into memory
    let sounds = load_sound_pack();
    let sounds = Arc::new(sounds);

    let pressed_keys: Arc<Mutex<HashSet<Key>>> = Arc::new(Mutex::new(HashSet::new()));

    // Create audio output ONCE — opening a device per-event is expensive (~50-200ms)
    let (_stream, stream_handle) =
        OutputStream::try_default().expect("Failed to open audio output");

    // Listen to global keyboard events
    if let Err(error) = listen(move |event: Event| {
        let action = match event.event_type {
            EventType::KeyPress(key) => {
                if pressed_keys.lock().unwrap().insert(key) {
                    Some((key, false))
                } else {
                    None
                }
            }
            EventType::KeyRelease(key) => {
                pressed_keys.lock().unwrap().remove(&key);
                Some((key, true))
            }
            _ => None,
        };

        if let Some((key, is_release)) = action {
            let sound_key = match key {
                Key::Backspace => {
                    if is_release { "backspace-up" } else { "backspace" }
                }
                Key::Return => {
                    if is_release { "enter-up" } else { "enter" }
                }
                Key::Space => {
                    if is_release { "spacebar-up" } else { "spacebar" }
                }
                _ => {
                    if is_release { "fallback-up" } else { "fallback" }
                }
            };

            if let Some(data) = sounds.get(sound_key) {
                let cursor = std::io::Cursor::new(data.as_ref().clone());
                if let Ok(source) = Decoder::new(cursor) {
                    let _ = stream_handle.play_raw(source.convert_samples());
                }
            }
        }
    }) {
        eprintln!("Error: {:?}", error);
    }
}

fn load_sound_pack() -> HashMap<String, Arc<Vec<u8>>> {
    let files = [
        "backspace", "backspace-up",
        "enter", "enter-up",
        "spacebar", "spacebar-up",
        "fallback", "fallback-up",
    ];

    let mut sounds = HashMap::new();
    for name in files {
        let path = format!("{PACK_DIR}/{name}.wav");
        match std::fs::read(&path) {
            Ok(data) => {
                sounds.insert(name.to_string(), Arc::new(data));
            }
            Err(e) => eprintln!("Warning: could not load {path}: {e}"),
        }
    }
    sounds
}
