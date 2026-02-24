//! cc-voice: Voice and Audio processing for cc-gateway
//!
//! This crate provides speech recognition (Whisper) and text-to-speech (TTS)
//! capabilities for voice-based channels.
//!
//! ## Features
//!
//! - **Speech Recognition**: OpenAI Whisper API and Groq Whisper API
//! - **Text-to-Speech**: OpenAI TTS and ElevenLabs API
//! - **Multiple formats**: MP3, Opus, AAC, FLAC, WAV
//! - **Voice channel support**: Foundation for Twilio Voice integration
//!
//! ## Usage
//!
//! ### Speech Recognition
//!
//! ```rust,ignore
//! use cc_voice::{WhisperClient, WhisperConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = WhisperConfig::openai("your-api-key");
//!     let client = WhisperClient::new(config)?;
//!
//!     let audio_data = std::fs::read("recording.mp3")?;
//!     let result = client.transcribe(&audio_data, "recording.mp3").await?;
//!
//!     println!("Transcription: {}", result.text);
//!     Ok(())
//! }
//! ```
//!
//! ### Text-to-Speech
//!
//! ```rust,ignore
//! use cc_voice::{TtsClient, TtsConfig, OpenAiVoice, AudioFormat};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = TtsConfig::openai("your-api-key")
//!         .with_voice("nova")
//!         .with_format(AudioFormat::Mp3);
//!
//!     let client = TtsClient::new(config)?;
//!     let result = client.synthesize("Hello, world!").await?;
//!
//!     std::fs::write("output.mp3", &result.audio_data)?;
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod tts;
pub mod whisper;

pub use error::{Result, VoiceError};
pub use tts::{
    AudioFormat, OpenAiVoice, SynthesisResult, TtsClient, TtsConfig, TtsProvider,
};
pub use whisper::{
    ResponseFormat, TranscriptionResult, TranscriptSegment, WhisperClient, WhisperConfig,
    WhisperProvider, WordSegment,
};
