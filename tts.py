import requests
import sounddevice as sd
import soundfile as sf
import io

KOKORO_URL = "http://localhost:8880/v1/audio/speech"


def speak(text):
    response = requests.post(
        KOKORO_URL,
        json={"model": "kokoro", "input": text, "voice": "af_heart", "response_format": "wav"}
    )
    response.raise_for_status()
    # Automatically play the raw audio stream locally
    data, fs = sf.read(io.BytesIO(response.content))
    sd.play(data, fs)
    sd.wait()
