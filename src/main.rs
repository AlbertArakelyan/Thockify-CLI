// Cargo.toml dependencies needed:
// [dependencies]
// rdev = "0.5"
// rodio = "0.17"

use rdev::{listen, Event, EventType, Key};
use rodio::{Decoder, OutputStream, Source};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

fn main() {
    println!("Starting global keyboard sound player...");
    println!("Press any key anywhere to hear sounds!");
    println!("Press Ctrl+C to exit");

    // Load the WAV file into memory once
    let sound_data = Arc::new(
        std::fs::read("click.wav")
            .expect("Failed to read click.wav - make sure it exists in the current directory"),
    );

    let pressed_keys: Arc<Mutex<HashSet<Key>>> = Arc::new(Mutex::new(HashSet::new()));

    // Create audio output ONCE — opening a device per-event is expensive (~50-200ms)
    let (_stream, stream_handle) =
        OutputStream::try_default().expect("Failed to open audio output");

    // Listen to global keyboard events
    if let Err(error) = listen(move |event: Event| {
        let should_play = match event.event_type {
            EventType::KeyPress(key) => pressed_keys.lock().unwrap().insert(key),
            EventType::KeyRelease(key) => pressed_keys.lock().unwrap().remove(&key),
            _ => false,
        };
        if should_play {
            let cursor = std::io::Cursor::new(sound_data.as_ref().clone());
            if let Ok(source) = Decoder::new(cursor) {
                // play_raw plays immediately and concurrently — no queuing
                let _ = stream_handle.play_raw(source.convert_samples());
            }
        }
    }) {
        eprintln!("Error: {:?}", error);
    }
}
