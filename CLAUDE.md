# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Cross-platform Rust utility that plays a click sound on every key press/release using global keyboard hooks. Single-file project (`src/main.rs`).

**Note:** Package name has a typo: `mech-keyboaurd-sound-rust` (extra 'u' in keyboard).

## Build & Run

```bash
cargo build              # debug build
cargo build --release    # release build
cargo run                # run (requires click.wav in cwd)
```

No tests exist currently. The project has no linter or formatter configured beyond standard `cargo fmt` / `cargo clippy`.

## Architecture

The entire application is in `src/main.rs` (~54 lines):

1. **Startup:** Loads `click.wav` into memory as `Arc<Vec<u8>>` for thread-safe sharing
2. **Event loop:** `rdev::listen()` blocks the main thread, capturing global keyboard events
3. **Audio playback:** Each KeyPress/KeyRelease spawns a new thread that decodes the WAV bytes via `rodio` and plays through the default audio output

Key dependencies:
- **rdev 0.5** — global keyboard event listener (uses winapi on Windows)
- **rodio 0.17** — audio decoding and playback (uses cpal for cross-platform audio)

## Runtime Requirements

- `click.wav` must be in the working directory
- Requires OS-level permission for global keyboard hooks
- Exit with Ctrl+C
