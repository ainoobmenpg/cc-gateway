//! Speech recognition using OpenAI Whisper API
//!
//! Supports multiple providers:
//! - OpenAI Whisper API
//! - Groq Whisper API (faster inference)
//! - Local Whisper models (via API)

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::error::{Result, VoiceError};

/// Whisper API provider
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhisperProvider {
    /// OpenAI Whisper API
    OpenAi,
    /// Groq Whisper API (faster)
    Groq,
    /// Custom API endpoint
    Custom(String),
}

/// Configuration for Whisper client
#[derive(Debug, Clone)]
pub struct WhisperConfig {
    /// API key
    pub api_key: String,
    /// Provider to use
    pub provider: WhisperProvider,
    /// Model to use (e.g., "whisper-1", "whisper-large-v3")
    pub model: String,
    /// Language hint (ISO 639-1 code, e.g., "en", "ja")
    pub language: Option<String>,
    /// Prompt to guide transcription
    pub prompt: Option<String>,
    /// Response format
    pub response_format: ResponseFormat,
    /// Temperature for sampling (0.0 - 1.0)
    pub temperature: Option<f32>,
}

impl WhisperConfig {
    /// Create a new OpenAI Whisper configuration
    pub fn openai(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            provider: WhisperProvider::OpenAi,
            model: "whisper-1".to_string(),
            language: None,
            prompt: None,
            response_format: ResponseFormat::Json,
            temperature: None,
        }
    }

    /// Create a new Groq Whisper configuration
    pub fn groq(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            provider: WhisperProvider::Groq,
            model: "whisper-large-v3".to_string(),
            language: None,
            prompt: None,
            response_format: ResponseFormat::Json,
            temperature: None,
        }
    }

    /// Set language hint
    pub fn with_language(mut self, lang: impl Into<String>) -> Self {
        self.language = Some(lang.into());
        self
    }

    /// Set prompt
    pub fn with_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    /// Get the API base URL for the provider
    pub fn base_url(&self) -> &str {
        match &self.provider {
            WhisperProvider::OpenAi => "https://api.openai.com/v1",
            WhisperProvider::Groq => "https://api.groq.com/openai/v1",
            WhisperProvider::Custom(url) => url,
        }
    }
}

/// Response format for transcription
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormat {
    #[default]
    Json,
    Text,
    Srt,
    VerboseJson,
    Vtt,
}

/// Transcription result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    /// Transcribed text
    pub text: String,
    /// Language detected (if available)
    pub language: Option<String>,
    /// Duration in seconds (if available)
    pub duration: Option<f64>,
    /// Word-level timestamps (if verbose_json format)
    pub words: Option<Vec<WordSegment>>,
    /// Segment-level timestamps (if verbose_json format)
    pub segments: Option<Vec<TranscriptSegment>>,
}

/// Word-level transcription segment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordSegment {
    pub word: String,
    pub start: f64,
    pub end: f64,
    pub probability: Option<f64>,
}

/// Segment-level transcription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptSegment {
    pub id: u32,
    pub start: f64,
    pub end: f64,
    pub text: String,
    pub tokens: Option<Vec<i32>>,
    pub temperature: Option<f64>,
    pub avg_logprob: Option<f64>,
    pub compression_ratio: Option<f64>,
    pub no_speech_prob: Option<f64>,
}

/// Whisper client for speech recognition
pub struct WhisperClient {
    client: Client,
    config: WhisperConfig,
}

impl WhisperClient {
    /// Create a new Whisper client
    pub fn new(config: WhisperConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| VoiceError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client, config })
    }

    /// Transcribe audio from bytes
    pub async fn transcribe(&self, audio_data: &[u8], filename: &str) -> Result<TranscriptionResult> {
        let url = format!("{}/audio/transcriptions", self.config.base_url());

        info!("Transcribing audio: {} bytes, filename: {}", audio_data.len(), filename);
        debug!("Using model: {}, provider: {:?}", self.config.model, self.config.provider);

        // Build multipart form
        let mut form = reqwest::multipart::Form::new()
            .text("model", self.config.model.clone())
            .text("response_format", "verbose_json".to_string())
            .part("file", reqwest::multipart::Part::bytes(audio_data.to_vec())
                .file_name(filename.to_string())
                .mime_str("audio/mpeg")
                .map_err(|e| VoiceError::EncodingError(format!("Failed to set mime type: {}", e)))?);

        if let Some(ref lang) = self.config.language {
            form = form.text("language", lang.clone());
        }

        if let Some(ref prompt) = self.config.prompt {
            form = form.text("prompt", prompt.clone());
        }

        if let Some(temp) = self.config.temperature {
            form = form.text("temperature", temp.to_string());
        }

        // Set authorization header based on provider
        let request = match &self.config.provider {
            WhisperProvider::OpenAi | WhisperProvider::Custom(_) => {
                self.client
                    .post(&url)
                    .header("Authorization", format!("Bearer {}", self.config.api_key))
                    .multipart(form)
            }
            WhisperProvider::Groq => {
                self.client
                    .post(&url)
                    .header("Authorization", format!("Bearer {}", self.config.api_key))
                    .multipart(form)
            }
        };

        let response = request
            .send()
            .await
            .map_err(|e| VoiceError::ApiError(format!("Request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(VoiceError::RecognitionFailed(format!(
                "API error {}: {}",
                status, error_text
            )));
        }

        // Parse response
        let result: TranscriptionResult = response.json().await.map_err(|e| {
            VoiceError::RecognitionFailed(format!("Failed to parse response: {}", e))
        })?;

        info!(
            "Transcription complete: {} characters, language: {:?}",
            result.text.len(),
            result.language
        );

        Ok(result)
    }

    /// Transcribe audio from base64 encoded data
    pub async fn transcribe_base64(&self, base64_audio: &str, filename: &str) -> Result<TranscriptionResult> {
        let audio_data = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, base64_audio)
            .map_err(|e| VoiceError::DecodingError(format!("Invalid base64: {}", e)))?;

        self.transcribe(&audio_data, filename).await
    }

    /// Transcribe audio with simple text response
    pub async fn transcribe_text(&self, audio_data: &[u8], filename: &str) -> Result<String> {
        let result = self.transcribe(audio_data, filename).await?;
        Ok(result.text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whisper_config_openai() {
        let config = WhisperConfig::openai("test-key");
        assert_eq!(config.provider, WhisperProvider::OpenAi);
        assert_eq!(config.model, "whisper-1");
        assert_eq!(config.base_url(), "https://api.openai.com/v1");
    }

    #[test]
    fn test_whisper_config_groq() {
        let config = WhisperConfig::groq("test-key");
        assert_eq!(config.provider, WhisperProvider::Groq);
        assert_eq!(config.model, "whisper-large-v3");
        assert_eq!(config.base_url(), "https://api.groq.com/openai/v1");
    }

    #[test]
    fn test_whisper_config_with_options() {
        let config = WhisperConfig::openai("test-key")
            .with_language("ja")
            .with_prompt("This is a Japanese conversation.");

        assert_eq!(config.language, Some("ja".to_string()));
        assert!(config.prompt.is_some());
    }

    #[test]
    fn test_response_format_default() {
        let format = ResponseFormat::default();
        assert_eq!(format, ResponseFormat::Json);
    }
}
