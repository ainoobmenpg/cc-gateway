//! WebSearch tool for searching the web via Exa API or DuckDuckGo

use async_trait::async_trait;
use cc_core::{Result, Tool, ToolResult};
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use std::env;

/// WebSearch tool for searching the web
pub struct WebSearchTool {
    client: Client,
    exa_api_key: Option<String>,
}

impl WebSearchTool {
    /// Create a new WebSearchTool instance
    pub fn new() -> Self {
        let exa_api_key = env::var("EXA_API_KEY").ok();
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { client, exa_api_key }
    }

    /// Create with custom client (for testing)
    pub fn with_client(client: Client) -> Self {
        let exa_api_key = env::var("EXA_API_KEY").ok();
        Self { client, exa_api_key }
    }
}

/// Search input parameters
#[derive(Debug, Deserialize)]
struct SearchInput {
    /// The search query
    query: String,
    /// Maximum number of results (default: 5)
    #[serde(default = "default_limit")]
    limit: usize,
    /// Use Exa API if available (default: true)
    #[serde(default = "default_true")]
    use_exa: bool,
}

fn default_limit() -> usize {
    5
}

fn default_true() -> bool {
    true
}

/// Exa API response structure
#[derive(Debug, Deserialize)]
struct ExaResponse {
    results: Vec<ExaResult>,
}

#[derive(Debug, Deserialize)]
struct ExaResult {
    title: Option<String>,
    url: String,
    published_date: Option<String>,
    #[serde(default)]
    text: String,
}

/// DuckDuckGo Instant Answer API response
#[derive(Debug, Deserialize)]
struct DuckDuckGoResponse {
    #[serde(default)]
    related_topics: Vec<DuckDuckGoTopic>,
    #[serde(default)]
    abstract_text: Option<String>,
    #[serde(default)]
    abstract_url: Option<String>,
    #[serde(default)]
    abstract_source: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DuckDuckGoTopic {
    text: Option<String>,
    first_url: Option<String>,
}

impl WebSearchTool {
    /// Search using Exa API
    async fn search_exa(&self, query: &str, limit: usize) -> Result<String> {
        let api_key = self.exa_api_key.as_ref().ok_or_else(|| {
            cc_core::Error::ToolExecution("EXA_API_KEY not configured".to_string())
        })?;

        let url = "https://api.exa.ai/search";

        let body = json!({
            "query": query,
            "numResults": limit,
            "useAutoprompt": true,
            "type": "auto"
        });

        let response = self
            .client
            .post(url)
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| cc_core::Error::ToolExecution(format!("Exa API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(cc_core::Error::ToolExecution(format!(
                "Exa API error ({}): {}",
                status, body
            )));
        }

        let exa_response: ExaResponse = response.json().await.map_err(|e| {
            cc_core::Error::ToolExecution(format!("Failed to parse Exa response: {}", e))
        })?;

        let results: Vec<SearchResult> = exa_response
            .results
            .into_iter()
            .map(|r| SearchResult {
                title: r.title.unwrap_or_else(|| "No title".to_string()),
                url: r.url,
                snippet: r.text,
                published_date: r.published_date,
            })
            .collect();

        Ok(format_results(&results, query))
    }

    /// Search using DuckDuckGo Instant Answer API
    async fn search_duckduckgo(&self, query: &str, _limit: usize) -> Result<String> {
        let url = format!(
            "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
            urlencoding::encode(query)
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                cc_core::Error::ToolExecution(format!("DuckDuckGo API request failed: {}", e))
            })?;

        if !response.status().is_success() {
            return Err(cc_core::Error::ToolExecution(format!(
                "DuckDuckGo API error: {}",
                response.status()
            )));
        }

        let ddg_response: DuckDuckGoResponse = response.json().await.map_err(|e| {
            cc_core::Error::ToolExecution(format!("Failed to parse DuckDuckGo response: {}", e))
        })?;

        let mut results = Vec::new();

        // Add abstract if available
        if let Some(abstract_text) = ddg_response.abstract_text {
            if !abstract_text.is_empty() {
                results.push(SearchResult {
                    title: ddg_response
                        .abstract_source
                        .unwrap_or_else(|| "Summary".to_string()),
                    url: ddg_response
                        .abstract_url
                        .unwrap_or_default(),
                    snippet: abstract_text,
                    published_date: None,
                });
            }
        }

        // Add related topics
        for topic in ddg_response.related_topics.iter().take(5) {
            if let (Some(text), Some(url)) = (&topic.text, &topic.first_url) {
                if !text.is_empty() {
                    results.push(SearchResult {
                        title: extract_title_from_text(text),
                        url: url.clone(),
                        snippet: text.clone(),
                        published_date: None,
                    });
                }
            }
        }

        if results.is_empty() {
            return Ok(format!(
                "No results found for '{}'. Try a different query.",
                query
            ));
        }

        Ok(format_results(&results, query))
    }
}

/// Internal search result structure
struct SearchResult {
    title: String,
    url: String,
    snippet: String,
    published_date: Option<String>,
}

/// Format search results for output
fn format_results(results: &[SearchResult], query: &str) -> String {
    let mut output = format!("Search results for: \"{}\"\n\n", query);

    for (i, result) in results.iter().enumerate() {
        output.push_str(&format!("## [{}] {}\n", i + 1, result.title));
        output.push_str(&format!("URL: {}\n", result.url));
        if let Some(date) = &result.published_date {
            output.push_str(&format!("Published: {}\n", date));
        }
        output.push_str(&format!("{}\n\n", result.snippet));
    }

    output.push_str(&format!(
        "Found {} results.\n",
        results.len()
    ));

    output
}

/// Extract title from DuckDuckGo topic text (usually in format "Title - Description")
fn extract_title_from_text(text: &str) -> String {
    text.split(" - ")
        .next()
        .unwrap_or("Result")
        .to_string()
}

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Search the web for information using Exa API (preferred) or DuckDuckGo fallback. Returns relevant search results with titles, URLs, and snippets."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query to look up"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of results to return (default: 5, max: 10)",
                    "minimum": 1,
                    "maximum": 10
                },
                "use_exa": {
                    "type": "boolean",
                    "description": "Use Exa API if available (default: true). Set to false to force DuckDuckGo."
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, input: Value) -> Result<ToolResult> {
        let search_input: SearchInput =
            serde_json::from_value(input).map_err(|e| {
                cc_core::Error::ToolExecution(format!("Invalid input parameters: {}", e))
            })?;

        if search_input.query.trim().is_empty() {
            return Ok(ToolResult::error("Query cannot be empty"));
        }

        let limit = search_input.limit.clamp(1, 10);

        tracing::info!(
            query = %search_input.query,
            limit = limit,
            use_exa = search_input.use_exa,
            "Executing web search"
        );

        // Try Exa first if enabled and API key is available
        if search_input.use_exa && self.exa_api_key.is_some() {
            tracing::debug!("Using Exa API for search");
            match self.search_exa(&search_input.query, limit).await {
                Ok(result) => return Ok(ToolResult::success(result)),
                Err(e) => {
                    tracing::warn!(error = %e, "Exa search failed, falling back to DuckDuckGo");
                    // Fall through to DuckDuckGo fallback
                }
            }
        }

        // Use DuckDuckGo as fallback or when explicitly requested
        tracing::debug!("Using DuckDuckGo for search");
        self.search_duckduckgo(&search_input.query, limit)
            .await
            .map(ToolResult::success)
    }
}

impl Default for WebSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_title_from_text() {
        let text = "Python - A programming language";
        assert_eq!(extract_title_from_text(text), "Python");
    }

    #[test]
    fn test_format_results() {
        let results = vec![SearchResult {
            title: "Test Title".to_string(),
            url: "https://example.com".to_string(),
            snippet: "Test snippet".to_string(),
            published_date: Some("2024-01-01".to_string()),
        }];

        let output = format_results(&results, "test query");
        assert!(output.contains("Test Title"));
        assert!(output.contains("https://example.com"));
        assert!(output.contains("2024-01-01"));
    }

    #[test]
    fn test_search_input_parsing() {
        let input = json!({
            "query": "rust programming",
            "limit": 3
        });

        let parsed: SearchInput = serde_json::from_value(input).unwrap();
        assert_eq!(parsed.query, "rust programming");
        assert_eq!(parsed.limit, 3);
        assert!(parsed.use_exa); // default
    }
}
