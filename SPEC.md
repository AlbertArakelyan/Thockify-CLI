# SPEC.md — Mechanical Keyboard Sound Simulator

## Overview

A cross-platform Rust application that simulates mechanical keyboard sounds in real time. Every physical key press and release triggers an authentic sound through the system audio, with zero perceptible latency regardless of typing speed.

## Current State

Single-file Rust project (`src/main.rs`, ~45 lines) that plays a single `click.wav` on every key press and release using global keyboard hooks.

### Dependencies

| Crate   | Version | Purpose                                      |
|---------|---------|----------------------------------------------|
| `rdev`  | 0.5     | Global keyboard event listener (winapi on Windows) |
| `rodio` | 0.17    | Audio decoding and playback (cpal backend)   |

### Architecture

1. **Startup** — Load audio data into memory as `Arc<Vec<u8>>`
2. **Event loop** — `rdev::listen()` blocks the main thread, capturing global `KeyPress`/`KeyRelease` events
3. **Audio playback** — `OutputStreamHandle::play_raw()` fires each sound immediately and concurrently through rodio's internal mixer
4. **Key tracking** — `HashSet<Key>` behind a `Mutex` prevents duplicate sounds from OS key-repeat

### Critical Design Decision: `play_raw()` over `Sink`

The simulator **must not** use `rodio::Sink` for playback. `Sink::append()` queues sounds sequentially — each waits for the previous to finish, causing growing latency during fast typing. `play_raw()` plays every sound at the exact moment of the input event, with full overlap support and no queuing. See `.claude/skills/lag-free-keyboard-audio.md` for the full rationale.

## Planned: Sound Pack System

The project will evolve from a single-sound player into a full mechanical keyboard simulator with swappable sound packs.

### Directory Structure

```
sound-packs/
├── cherry-mx-blue/
│   ├── ctrl-down.wav
│   ├── ctrl-up.wav
│   ├── shift-down.wav
│   ├── shift-up.wav
│   ├── space-down.wav
│   ├── space-up.wav
│   ├── enter-down.wav
│   ├── enter-up.wav
│   ├── backspace-down.wav
│   ├── backspace-up.wav
│   ├── tab-down.wav
│   ├── tab-up.wav
│   ├── alt-down.wav
│   ├── alt-up.wav
│   ├── escape-down.wav
│   ├── escape-up.wav
│   ├── fallback-down.wav      # all other keys (press)
│   └── fallback-up.wav        # all other keys (release)
├── cherry-mx-brown/
│   ├── ctrl-down.wav
│   ├── ctrl-up.wav
│   ├── ...
│   ├── fallback-down.wav
│   └── fallback-up.wav
├── cherry-mx-red/
│   └── ...
└── topre-45g/
    └── ...
```

### Naming Convention

- **`<key>-down.wav`** — sound for key press
- **`<key>-up.wav`** — sound for key release
- **`fallback-down.wav`** — default press sound for any key without a dedicated file (required)
- **`fallback-up.wav`** — default release sound for any key without a dedicated file (required)

### Sound Pack Loading

At startup (or on pack switch), the simulator will:

1. Scan the selected pack directory for all `*-down.wav` and `*-up.wav` files
2. Load each file into memory as `Arc<Vec<u8>>` (pre-loaded, not read from disk per event)
3. Build a lookup map: `HashMap<(Key, Direction), Arc<Vec<u8>>>` where `Direction` is `Down` or `Up`
4. On each key event, look up the specific key sound; if not found, use `fallback-down.wav` or `fallback-up.wav`

### Key-to-Filename Mapping

| Key(s)          | Filename prefix |
|-----------------|-----------------|
| Ctrl (L/R)      | `ctrl`          |
| Shift (L/R)     | `shift`         |
| Alt (L/R)       | `alt`           |
| Space           | `space`         |
| Enter/Return    | `enter`         |
| Backspace       | `backspace`     |
| Tab             | `tab`           |
| Escape          | `escape`        |
| Caps Lock       | `capslock`      |
| Everything else | uses `fallback` |

This table will grow as packs provide more granular sounds (e.g., `a-down.wav` for individual letter keys).

### Pack Selection

The active sound pack will be selected by name at startup (e.g., via CLI argument or config). The `click.wav` in the project root will be removed once the sound pack system is implemented.

## Runtime Requirements

- `sound-packs/` directory with at least one pack containing `fallback-down.wav` and `fallback-up.wav`
- OS-level permission for global keyboard hooks
- Audio output device available
- Exit with Ctrl+C

## Platform Support

| Platform | Status  | Hook backend |
|----------|---------|--------------|
| Windows  | Primary | winapi       |
| macOS    | Planned | CGEventTap   |
| Linux    | Planned | X11/evdev    |

All platforms use the same `rdev` + `rodio` stack. Platform differences are handled internally by these crates.
