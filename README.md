# Kokoplay

A small local HTTP service that turns text into speech and plays it out loud on the machine it runs on. It sends text to a local [Kokoro](https://github.com/remsky/Kokoro-FastAPI)-compatible TTS backend and plays the resulting audio automatically through the default output device.

## Prerequisites

- A Rust toolchain (`rustc`/`cargo`, via [rustup](https://rustup.rs/))
- A Kokoro-compatible TTS server running locally and reachable at `http://localhost:8880/v1/audio/speech` (the OpenAI-style `/v1/audio/speech` endpoint). Override with `--kokoro-url` if yours runs elsewhere.
- A working audio output device on the host, since playback happens locally via `rodio`.

## Install

```
cargo build --release
```

This produces a single, self-contained `target\release\kokoplay.exe` — no Python, venv, or pip involved, and no other runtime files need to travel with it.

## Run

Start the Kokoro backend first, then start the service:

```
.\target\release\kokoplay.exe
```

The service listens on `http://127.0.0.1:8000` by default. Override defaults with flags:

```
.\target\release\kokoplay.exe --host 127.0.0.1 --port 8000 --kokoro-url http://localhost:8880/v1/audio/speech --voice af_heart
```

To speak one line without starting the server:

```
.\target\release\kokoplay.exe speak "System notification: task completed successfully."
```

## Usage

**Speak some text** — queues it for playback and returns immediately (`202 Accepted`); a background worker plays queued text one at a time so requests never overlap:

```
curl -X POST http://127.0.0.1:8000/speak -H "Content-Type: application/json" -d "{\"text\": \"hello world\"}"
```

**Health check**:

```
curl http://127.0.0.1:8000/health
```

## Project structure

| File | Purpose |
|---|---|
| `Cargo.toml` | Rust package manifest and dependencies. |
| `src/main.rs` | Entry point — dispatches to the `serve` or `speak` subcommand. |
| `src/cli.rs` | CLI flag/subcommand parsing (`clap`), with defaults matching the values above. |
| `src/kokoro.rs` | HTTP client for the Kokoro backend's `/v1/audio/speech` endpoint. |
| `src/audio.rs` | WAV decoding (including a fix for Kokoro's streamed/unknown-size WAV headers) and local playback via `rodio`. |
| `src/tts.rs` | Combines the Kokoro client and audio playback into a single `speak(text)` call. |
| `src/worker.rs` | Background thread + queue that plays queued text one at a time so requests never overlap. |
| `src/http.rs` | Axum app exposing `/speak` and `/health` over HTTP. |
