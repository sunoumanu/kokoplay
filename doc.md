Implementation complete and verified via HTTP (all requests returned correct status codes, no playback errors logged). Since I can't hear audio myself, please confirm you heard the test phrases play (should have queued/spoken "Hello, this is a test...", then "First message, one." followed by "Second message, two.").



What changed:

\- tts.py (new) — the shared speak() function, extracted from run.py, now with response.raise\_for\_status() so a dead Kokoro backend fails loudly.

\- run.py — now just imports speak and calls the demo line under if \_\_name\_\_ == "\_\_main\_\_":.

\- server.py (new) — FastAPI app with POST /speak (queues text, returns 202 immediately) and GET /health. A background daemon thread plays queued text one at a time via speak(), so concurrent requests never overlap audio.

\- Installed fastapi and uvicorn into .venv.



To run it:

.venv\\Scripts\\python.exe -m uvicorn server:a00

Then POST to it from anywhere:

curl -X POST http://127.0.0.1:8000/speak -H on" -d "{\\"text\\": \\"hello\\"}"



Let me know if you heard the test audio play correctly, or if you'd like a requirements.txt, a different port, or the server set up to auto-start (e.g. as a Windows service/scheduled task).

