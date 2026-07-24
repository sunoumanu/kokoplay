# Kokoplay

A small local HTTP service that turns text into speech and plays it out loud on the machine it runs on. It sends text to a local [Kokoro](https://github.com/remsky/Kokoro-FastAPI)-compatible TTS backend and plays the resulting audio automatically through the default output device.

## Prerequisites

- Python 3.11+ (developed with 3.14)
- A Kokoro-compatible TTS server running locally and reachable at `http://localhost:8880/v1/audio/speech` (the OpenAI-style `/v1/audio/speech` endpoint). Change `KOKORO_URL` in `tts.py` if yours runs elsewhere.
- A working audio output device on the host, since playback happens locally via `sounddevice`.

## Install

```
python -m venv .venv
.venv\Scripts\activate
pip install -r requirements.txt
```

There's no compilation step — this is pure Python, so "build" is just setting up the virtual environment and installing dependencies above.

## Run

Start the Kokoro backend first, then start the service:

```
.venv\Scripts\python.exe -m uvicorn server:app --host 127.0.0.1 --port 8000
```

The service listens on `http://127.0.0.1:8000`.

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
| `tts.py` | Shared `speak(text)` function — calls the Kokoro backend and plays the returned audio. |
| `server.py` | FastAPI app exposing `/speak` and `/health` over HTTP, with a queued background playback worker. |
| `run.py` | Standalone demo script — run directly (`python run.py`) to speak one hardcoded line without starting the server. |
| `requirements.txt` | Python dependencies. |
