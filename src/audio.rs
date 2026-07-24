use std::io::Cursor;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("failed to open audio output device: {0}")]
    Device(#[from] rodio::StreamError),
    #[error("failed to create audio sink: {0}")]
    Sink(#[from] rodio::PlayError),
    #[error("failed to decode wav audio: {0}")]
    Decode(#[from] rodio::decoder::DecoderError),
}

/// Kokoro streams its WAV response through ffmpeg, which writes the RIFF and
/// `data` chunk sizes as 0xFFFFFFFF ("unknown/streamed length") instead of
/// patching in the real size once the stream ends. libsndfile (used by the
/// Python version) tolerates this; rodio's WAV decoder does not. Since we
/// always have the full response buffered already, rewrite both size fields
/// to their real values before decoding.
fn patch_wav_header(bytes: &mut [u8]) {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return;
    }

    let riff_size = (bytes.len() - 8) as u32;
    bytes[4..8].copy_from_slice(&riff_size.to_le_bytes());

    let mut pos = 12;
    while pos + 8 <= bytes.len() {
        let tag = &bytes[pos..pos + 4];
        let declared_size = u32::from_le_bytes(bytes[pos + 4..pos + 8].try_into().unwrap());

        if tag == b"data" {
            let actual_size = (bytes.len() - pos - 8) as u32;
            bytes[pos + 4..pos + 8].copy_from_slice(&actual_size.to_le_bytes());
            break;
        }

        if declared_size == u32::MAX {
            break;
        }

        let advance = declared_size as usize + (declared_size as usize % 2);
        pos += 8 + advance;
    }
}

pub fn decode_wav_bytes(
    mut bytes: Vec<u8>,
) -> Result<rodio::Decoder<Cursor<Vec<u8>>>, AudioError> {
    patch_wav_header(&mut bytes);
    Ok(rodio::Decoder::new(Cursor::new(bytes))?)
}

pub fn play_blocking(bytes: Vec<u8>) -> Result<(), AudioError> {
    let source = decode_wav_bytes(bytes)?;
    let (_stream, handle) = rodio::OutputStream::try_default()?;
    let sink = rodio::Sink::try_new(&handle)?;
    sink.append(source);
    sink.sleep_until_end();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_wav(streamed_sizes: bool, pcm: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&(if streamed_sizes { u32::MAX } else { (36 + pcm.len()) as u32 }).to_le_bytes());
        out.extend_from_slice(b"WAVE");
        out.extend_from_slice(b"fmt ");
        out.extend_from_slice(&16u32.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes()); // PCM
        out.extend_from_slice(&1u16.to_le_bytes()); // mono
        out.extend_from_slice(&24000u32.to_le_bytes()); // sample rate
        out.extend_from_slice(&48000u32.to_le_bytes()); // byte rate
        out.extend_from_slice(&2u16.to_le_bytes()); // block align
        out.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
        out.extend_from_slice(b"data");
        out.extend_from_slice(&(if streamed_sizes { u32::MAX } else { pcm.len() as u32 }).to_le_bytes());
        out.extend_from_slice(pcm);
        out
    }

    #[test]
    fn decodes_conventional_wav() {
        let pcm = [0u8; 400];
        let wav = make_wav(false, &pcm);
        assert!(decode_wav_bytes(wav).is_ok());
    }

    #[test]
    fn decodes_kokoro_shaped_streamed_wav() {
        let pcm = [0u8; 400];
        let wav = make_wav(true, &pcm);
        assert!(decode_wav_bytes(wav).is_ok());
    }

    #[test]
    fn patch_wav_header_fixes_sizes() {
        let pcm = [1u8, 2, 3, 4, 5];
        let mut wav = make_wav(true, &pcm);
        let riff_before = u32::from_le_bytes(wav[4..8].try_into().unwrap());
        assert_eq!(riff_before, u32::MAX);
        patch_wav_header(&mut wav);
        let riff_after = u32::from_le_bytes(wav[4..8].try_into().unwrap());
        assert_eq!(riff_after, (wav.len() - 8) as u32);
        let data_size = u32::from_le_bytes(wav[wav.len() - pcm.len() - 4..wav.len() - pcm.len()].try_into().unwrap());
        assert_eq!(data_size, pcm.len() as u32);
    }
}
