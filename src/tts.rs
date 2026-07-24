use thiserror::Error;

use crate::audio;
use crate::kokoro::{KokoroClient, KokoroError};

#[derive(Debug, Error)]
pub enum SpeakError {
    #[error(transparent)]
    Kokoro(#[from] KokoroError),
    #[error(transparent)]
    Audio(#[from] audio::AudioError),
}

pub fn speak(client: &KokoroClient, text: &str) -> Result<(), SpeakError> {
    let wav_bytes = client.synthesize(text)?;
    audio::play_blocking(wav_bytes)?;
    Ok(())
}
