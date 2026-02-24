//! Text-to-Speech synthesis
//!
//! Supports multiple providers:
//! - OpenAI TTS API
//! - ElevenLabs API
//! - Google Cloud TTS

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::error::{Result, VoiceError};

/// TTS API provider
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TtsProvider {
    /// OpenAI TTS API
    OpenAi,
    /// ElevenLabs API
    ElevenLabs,
    /// Custom API endpoint
    Custom(String),
}

/// Available OpenAI TTS voices
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OpenAiVoice {
    #[default]
    Alloy,
    Echo,
    Fable,
    Onyx,
    Nova,
    Shimmer,
}

impl std::fmt::Display for OpenAiVoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Alloy => write!(f, "alloy"),
            Self::Echo => write!(f, "echo"),
            Self::Fable => write!(f, "fable"),
            Self::Onyx => write!(f, "onyx"),
            Self::Nova => write!(f, "nova"),
            Self::Shimmer => write!(f, "shimmer"),
        }
    }
}

/// TTS configuration
#[derive(Debug, Clone)]
pub struct TtsConfig {
    /// API key
    pub api_key: String,
    /// Provider to use
    pub provider: TtsProvider,
    /// Model to use
    pub model: String,
    /// Voice to use
    pub voice: String,
    /// Response format (audio format)
    pub response_format: AudioFormat,
    /// Speech speed (0.25 - 4.0)
    pub speed: Option<f32>,
    /// ElevenLabs voice ID (if using ElevenLabs)
    pub voice_id: Option<String>,
}

impl TtsConfig {
    /// Create a new OpenAI TTS configuration
    pub fn openai(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            provider: TtsProvider::OpenAi,
            model: "tts-1".to_string(),
            voice: "alloy".to_string(),
            response_format: AudioFormat::Mp3,
            speed: None,
            voice_id: None,
        }
    }

    /// Create a new OpenAI TTS HD configuration
    pub fn openai_hd(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            provider: TtsProvider::OpenAi,
            model: "tts-1-hd".to_string(),
            voice: "alloy".to_string(),
            response_format: AudioFormat::Mp3,
            speed: None,
            voice_id: None,
        }
    }

    /// Create a new ElevenLabs TTS configuration
    pub fn elevenlabs(api_key: impl Into<String>, voice_id: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            provider: TtsProvider::ElevenLabs,
            model: "eleven_monolingual_v1".to_string(),
            voice: "default".to_string(),
            response_format: AudioFormat::Mp3,
            speed: None,
            voice_id: Some(voice_id.into()),
        }
    }

    /// Set voice
    pub fn with_voice(mut self, voice: impl Into<String>) -> Self {
        self.voice = voice.into();
        self
    }

    /// Set speed
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = Some(speed.clamp(0.25, 4.0));
        self
    }

    /// Set format
    pub fn with_format(mut self, format: AudioFormat) -> Self {
        self.response_format = format;
        self
    }

    /// Get the API base URL for the provider
    pub fn base_url(&self) -> &str {
        match &self.provider {
            TtsProvider::OpenAi => "https://api.openai.com/v1",
            TtsProvider::ElevenLabs => "https://api.elevenlabs.io/v1",
            TtsProvider::Custom(url) => url,
        }
    }
}

/// Audio format for TTS output
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioFormat {
    #[default]
    Mp3,
    Opus,
    Aac,
    Flac,
    Wav,
    Pcm,
}

impl std::fmt::Display for AudioFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mp3 => write!(f, "mp3"),
            Self::Opus => write!(f, "opus"),
            Self::Aac => write!(f, "aac"),
            Self::Flac => write!(f, "flac"),
            Self::Wav => write!(f, "wav"),
            Self::Pcm => write!(f, "pcm"),
        }
    }
}

/// TTS synthesis result
#[derive(Debug, Clone)]
pub struct SynthesisResult {
    /// Audio data (encoded in the requested format)
    pub audio_data: Vec<u8>,
    /// Audio format
    pub format: AudioFormat,
    /// Duration in seconds (if available)
    pub duration: Option<f64>,
    /// Content type
    pub content_type: String,
}

impl SynthesisResult {
    /// Get audio data as base64
    pub fn to_base64(&self) -> String {
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &self.audio_data)
    }
}

/// TTS client for speech synthesis
pub struct TtsClient {
    client: Client,
    config: TtsConfig,
}

impl TtsClient {
    /// Create a new TTS client
    pub fn new(config: TtsConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| VoiceError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client, config })
    }

    /// Synthesize speech from text
    pub async fn synthesize(&self, text: &str) -> Result<SynthesisResult> {
        match &self.config.provider {
            TtsProvider::OpenAi => self.synthesize_openai(text).await,
            TtsProvider::ElevenLabs => self.synthesize_elevenlabs(text).await,
            TtsProvider::Custom(_) => self.synthesize_openai(text).await, // Assume OpenAI-compatible
        }
    }

    /// Synthesize using OpenAI TTS API
    async fn synthesize_openai(&self, text: &str) -> Result<SynthesisResult> {
        let url = format!("{}/audio/speech", self.config.base_url());

        info!("Synthesizing speech: {} chars using OpenAI", text.len());
        debug!("Model: {}, Voice: {}", self.config.model, self.config.voice);

        let mut body = serde_json::json!({
            "model": self.config.model,
            "input": text,
            "voice": self.config.voice,
            "response_format": self.config.response_format.to_string(),
        });

        if let Some(speed) = self.config.speed {
            body["speed"] = serde_json::json!(speed);
        }

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| VoiceError::ApiError(format!("Request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(VoiceError::SynthesisFailed(format!(
                "API error {}: {}",
                status, error_text
            )));
        }

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("audio/mpeg")
            .to_string();

        let audio_data = response.bytes().await.map_err(|e| {
            VoiceError::SynthesisFailed(format!("Failed to read audio data: {}", e))
        })?;

        info!(
            "Synthesis complete: {} bytes, content-type: {}",
            audio_data.len(),
            content_type
        );

        Ok(SynthesisResult {
            audio_data: audio_data.to_vec(),
            format: self.config.response_format,
            duration: None,
            content_type,
        })
    }

    /// Synthesize using ElevenLabs API
    async fn synthesize_elevenlabs(&self, text: &str) -> Result<SynthesisResult> {
        let voice_id = self.config.voice_id.as_ref().ok_or_else(|| {
            VoiceError::ConfigError("ElevenLabs requires voice_id".to_string())
        })?;

        let url = format!(
            "{}/text-to-speech/{}",
            self.config.base_url(),
            voice_id
        );

        info!("Synthesizing speech: {} chars using ElevenLabs", text.len());

        let body = serde_json::json!({
            "text": text,
            "model_id": self.config.model,
            "voice_settings": {
                "stability": 0.5,
                "similarity_boost": 0.75,
            }
        });

        let response = self
            .client
            .post(&url)
            .header("xi-api-key", &self.config.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| VoiceError::ApiError(format!("Request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(VoiceError::SynthesisFailed(format!(
                "API error {}: {}",
                status, error_text
            )));
        }

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("audio/mpeg")
            .to_string();

        let audio_data = response.bytes().await.map_err(|e| {
            VoiceError::SynthesisFailed(format!("Failed to read audio data: {}", e))
        })?;

        info!(
            "Synthesis complete: {} bytes, content-type: {}",
            audio_data.len(),
            content_type
        );

        Ok(SynthesisResult {
            audio_data: audio_data.to_vec(),
            format: self.config.response_format,
            duration: None,
            content_type,
        })
    }

    /// Synthesize and return as base64
    pub async fn synthesize_base64(&self, text: &str) -> Result<String> {
        let result = self.synthesize(text).await?;
        Ok(result.to_base64())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tts_config_openai() {
        let config = TtsConfig::openai("test-key");
        assert_eq!(config.provider, TtsProvider::OpenAi);
        assert_eq!(config.model, "tts-1");
        assert_eq!(config.base_url(), "https://api.openai.com/v1");
    }

    #[test]
    fn test_tts_config_openai_hd() {
        let config = TtsConfig::openai_hd("test-key");
        assert_eq!(config.model, "tts-1-hd");
    }

    #[test]
    fn test_tts_config_elevenlabs() {
        let config = TtsConfig::elevenlabs("test-key", "voice-123");
        assert_eq!(config.provider, TtsProvider::ElevenLabs);
        assert_eq!(config.voice_id, Some("voice-123".to_string()));
    }

    #[test]
    fn test_tts_config_with_options() {
        let config = TtsConfig::openai("test-key")
            .with_voice("nova")
            .with_speed(1.5)
            .with_format(AudioFormat::Opus);

        assert_eq!(config.voice, "nova");
        assert_eq!(config.speed, Some(1.5));
        assert_eq!(config.response_format, AudioFormat::Opus);
    }

    #[test]
    fn test_openai_voice_display() {
        assert_eq!(OpenAiVoice::Alloy.to_string(), "alloy");
        assert_eq!(OpenAiVoice::Nova.to_string(), "nova");
    }

    #[test]
    fn test_audio_format_display() {
        assert_eq!(AudioFormat::Mp3.to_string(), "mp3");
        assert_eq!(AudioFormat::Opus.to_string(), "opus");
    }

    #[test]
    fn test_synthesis_result_to_base64() {
        let result = SynthesisResult {
            audio_data: b"test audio data".to_vec(),
            format: AudioFormat::Mp3,
            duration: Some(5.0),
            content_type: "audio/mpeg".to_string(),
        };

        let base64 = result.to_base64();
        assert!(!base64.is_empty());
    }
}
