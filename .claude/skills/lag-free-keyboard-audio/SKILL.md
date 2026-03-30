---
name: lag-free-keyboard-audio
description: Guide for implementing zero-latency concurrent audio playback with rodio. Use when working on sound playback, fixing audio lag, or adding new key-sound features.
user-invocable: false
---

# Lag-Free Mechanical Keyboard Sound Playback in Rust

## Problem

Playing a sound on every key press/release using `rodio` with a `Sink` causes sounds to **queue sequentially**. When keys are pressed rapidly or simultaneously, each sound waits for the previous one to finish, creating noticeable lag that grows with each keystroke. The audio falls behind the actual input, and continues playing long after typing stops.

### Root Cause

`rodio::Sink::append()` is a FIFO queue — it serializes all audio. Sound B doesn't start until sound A finishes. This is fundamentally wrong for input feedback where each event must produce immediate, overlapping audio.

## Solution

Use `OutputStreamHandle::play_raw()` instead of `Sink::append()`. This plays each sound **immediately and concurrently** through rodio's internal mixer, with zero queuing.

### Key Principles

1. **Create `OutputStream` once at startup** — opening an audio device is expensive (~50-200ms). Never create it per-event.
2. **Use `play_raw()` for concurrent playback** — each call starts a new sound immediately, overlapping with any already playing. No queue, no waiting.
3. **Load audio data into memory once** — read the WAV file into an `Arc<Vec<u8>>` at startup. Clone the bytes (not the file handle) for each event via `Cursor::new()`.
4. **Call `.convert_samples()`** — `play_raw()` requires `Source<Item = f32>`, so convert the decoded samples before passing them in.

### Pattern

```rust
use rodio::{Decoder, OutputStream, Source};

// At startup — create output device once
let (_stream, stream_handle) = OutputStream::try_default().expect("Failed to open audio output");

// Load sound data once
let sound_data = Arc::new(std::fs::read("click.wav").expect("Failed to read click.wav"));

// On each key event — play immediately, concurrently
let cursor = std::io::Cursor::new(sound_data.as_ref().clone());
if let Ok(source) = Decoder::new(cursor) {
    let _ = stream_handle.play_raw(source.convert_samples());
}
```

### What NOT to Do

- **Don't use `Sink`** for real-time input feedback — it queues sounds sequentially.
- **Don't create `OutputStream` per event** — the device open cost causes ~50-200ms latency spikes.
- **Don't spawn threads per sound** — `play_raw()` handles concurrency internally via the mixer; spawning threads adds overhead and complexity for no benefit.
- **Don't use `Sink::try_new()` per event either** — while less expensive than opening a device, it still adds unnecessary overhead compared to `play_raw()`.

## When to Apply

Any time you need instant, overlapping audio feedback tied to high-frequency input events (keyboard, mouse, game controller, MIDI). The `play_raw()` approach scales to dozens of concurrent sounds without lag.
