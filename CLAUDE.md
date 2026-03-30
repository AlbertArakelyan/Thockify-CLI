# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Mechanical keyboard sound simulator in Rust. Plays per-key sounds on every key press/release via global keyboard hooks with zero-latency concurrent audio. Currently a single-file project (`src/main.rs`) evolving toward a full sound pack system (see `SPEC.md`).

**Note:** Package name has a typo in Cargo.toml: `mech-keyboaurd-sound-rust` (extra 'u' in keyboard).

## Build & Run

```bash
cargo build              # debug build
cargo build --release    # release build
cargo run                # run (requires click.wav in cwd)
cargo fmt                # format
cargo clippy             # lint
```

No tests exist. The app requires OS-level permission for global keyboard hooks and an audio output device. Exit with Ctrl+C.

## Architecture

Everything is in `src/main.rs` (~45 lines):

1. **Startup** — Loads `click.wav` into `Arc<Vec<u8>>`, opens one `OutputStream`
2. **Event loop** — `rdev::listen()` blocks main thread, captures global `KeyPress`/`KeyRelease`
3. **Key dedup** — `HashSet<Key>` behind `Mutex` filters out OS key-repeat duplicates
4. **Audio** — `stream_handle.play_raw(source.convert_samples())` plays each sound instantly and concurrently

### Critical: Never use `rodio::Sink` for playback

`Sink::append()` queues sounds sequentially — each waits for the previous to finish, causing growing latency during fast typing. Always use `play_raw()` for immediate concurrent playback. Full rationale in `.claude/skills/lag-free-keyboard-audio.md`.

## Dependencies

- **rdev 0.5** — global keyboard event listener (winapi on Windows)
- **rodio 0.17** — audio decoding/playback (cpal backend)

## Planned Direction

Evolving from single `click.wav` to a `sound-packs/` system with per-key WAV files (`<key>-down.wav`, `<key>-up.wav`) and `fallback-down.wav`/`fallback-up.wav` for unspecified keys. Full design in `SPEC.md`.
