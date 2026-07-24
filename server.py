import queue
import threading
from contextlib import asynccontextmanager

from fastapi import FastAPI, HTTPException
from pydantic import BaseModel

from tts import speak

text_queue: "queue.Queue[str]" = queue.Queue()


def _worker():
    while True:
        text = text_queue.get()
        try:
            speak(text)
        except Exception as e:
            print(f"TTS playback failed: {e}")
        finally:
            text_queue.task_done()


@asynccontextmanager
async def lifespan(app: FastAPI):
    threading.Thread(target=_worker, daemon=True).start()
    yield


app = FastAPI(lifespan=lifespan)


class SpeakRequest(BaseModel):
    text: str


@app.post("/speak", status_code=202)
def speak_endpoint(req: SpeakRequest):
    if not req.text.strip():
        raise HTTPException(status_code=400, detail="text must not be empty")
    text_queue.put(req.text)
    return {"status": "queued", "queue_length": text_queue.qsize()}


@app.get("/health")
def health():
    return {"status": "ok"}
