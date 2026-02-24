//! WebFetch tool for fetching and parsing web content

use async_trait::async_trait;
use cc_core::{Result, Tool, ToolResult};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::Deserialize;
use serde_json::{json, Value};
use std::env;

/// WebFetch tool for fetching and parsing web pages
pub struct WebFetchTool {
    client: Client,
    max_content_length: usize,
}

impl WebFetchTool {
    /// Create a new WebFetchTool instance
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (compatible; cc-gateway/0.1)")
            .build()
            .unwrap_or_else(|_| Client::new());

        let max_content_length = env::var("WEB_FETCH_MAX_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1_000_000); // 1MB default

        Self {
            client,
            max_content_length,
        }
    }

    /// Create with custom client (for testing)
    pub fn with_client(client: Client) -> Self {
        let max_content_length = env::var("WEB_FETCH_MAX_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1_000_000);
        Self {
            client,
            max_content_length,
        }
    }
}

/// Fetch input parameters
#[derive(Debug, Deserialize)]
struct FetchInput {
    /// The URL to fetch
    url: String,
    /// Extract main content only (default: true)
    #[serde(default = "default_true")]
    extract_main: bool,
    /// Include links in output (default: false)
    #[serde(default)]
    include_links: bool,
    /// Maximum characters to return (default: 10000)
    #[serde(default = "default_max_chars")]
    max_chars: usize,
}

fn default_max_chars() -> usize {
    10000
}

fn default_true() -> bool {
    true
}

impl WebFetchTool {
    /// Fetch and parse a web page
    async fn fetch_url(&self, url: &str, extract_main: bool, include_links: bool, max_chars: usize) -> Result<String> {
        // Validate URL
        let parsed_url = url::Url::parse(url)
            .map_err(|e| cc_core::Error::ToolExecution(format!("Invalid URL: {}", e)))?;

        // Only allow http/https
        if !matches!(parsed_url.scheme(), "http" | "https") {
            return Err(cc_core::Error::ToolExecution(
                "Only HTTP and HTTPS URLs are supported".to_string(),
            ));
        }

        tracing::info!(url = %url, "Fetching web page");

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| cc_core::Error::ToolExecution(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(cc_core::Error::ToolExecution(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        // Check content length
        if let Some(content_length) = response.content_length() {
            if content_length as usize > self.max_content_length {
                return Err(cc_core::Error::ToolExecution(format!(
                    "Content too large: {} bytes (max: {})",
                    content_length, self.max_content_length
                )));
            }
        }

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "text/html".to_string());

        // Check if HTML
        if !content_type.contains("text/html") && !content_type.contains("application/xhtml") {
            // Return raw content for non-HTML
            let text = response.text().await.map_err(|e| {
                cc_core::Error::ToolExecution(format!("Failed to read response: {}", e))
            })?;
            return Ok(format!("Content-Type: {}\n\n{}", content_type, text));
        }

        let html_content = response.text().await.map_err(|e| {
            cc_core::Error::ToolExecution(format!("Failed to read response: {}", e))
        })?;

        // Parse HTML
        let document = Html::parse_document(&html_content);

        // Extract title
        let title = extract_title(&document);

        // Extract content
        let content = if extract_main {
            extract_main_content(&document)
        } else {
            extract_all_text(&document)
        };

        // Extract links if requested
        let links_section = if include_links {
            extract_links(&document, &parsed_url)
        } else {
            String::new()
        };

        // Truncate if needed
        let truncated_content = truncate_text(&content, max_chars);
        let was_truncated = truncated_content.len() < content.len();

        // Format output
        let mut output = format!("Title: {}\n\n", title);
        output.push_str(&truncated_content);

        if was_truncated {
            output.push_str(&format!(
                "\n\n[Content truncated to {} characters]",
                max_chars
            ));
        }

        if !links_section.is_empty() {
            output.push_str("\n\n## Links\n");
            output.push_str(&links_section);
        }

        Ok(output)
    }
}

/// Extract the page title
fn extract_title(document: &Html) -> String {
    let title_selector = Selector::parse("title").ok();
    if let Some(selector) = title_selector {
        if let Some(title_elem) = document.select(&selector).next() {
            return title_elem.text().collect::<String>().trim().to_string();
        }
    }
    "No title".to_string()
}

/// Extract main content (article, main, or body)
fn extract_main_content(document: &Html) -> String {
    // Try common content selectors in order of preference
    let content_selectors = ["article", "main", "[role='main']", ".content", "#content", "body"];

    for selector_str in &content_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            let content: String = document
                .select(&selector)
                .flat_map(|elem| elem.text())
                .collect::<String>()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");

            if content.len() > 100 {
                return content;
            }
        }
    }

    // Fallback to all text
    extract_all_text(document)
}

/// Extract all text content from the document
fn extract_all_text(document: &Html) -> String {
    // Remove script and style elements first
    let body_selector = Selector::parse("body").ok();

    if let Some(selector) = body_selector {
        document
            .select(&selector)
            .next()
            .map(|body| {
                body.text()
                    .collect::<String>()
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default()
    } else {
        document
            .root_element()
            .text()
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Extract links from the document
fn extract_links(document: &Html, base_url: &url::Url) -> String {
    let link_selector = match Selector::parse("a[href]") {
        Ok(s) => s,
        Err(_) => return String::new(),
    };

    let mut links = Vec::new();
    for link in document.select(&link_selector).take(20) {
        if let Some(href) = link.value().attr("href") {
            let text: String = link.text().collect();
            let text = text.split_whitespace().collect::<Vec<_>>().join(" ");

            // Resolve relative URLs
            let full_url = if href.starts_with("http://") || href.starts_with("https://") {
                href.to_string()
            } else {
                match base_url.join(href) {
                    Ok(resolved) => resolved.to_string(),
                    Err(_) => href.to_string(),
                }
            };

            if !text.is_empty() && !full_url.starts_with("javascript:") {
                links.push(format!("- [{}]({})", text, full_url));
            }
        }
    }

    links.join("\n")
}

/// Truncate text to max characters
fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
        return text.to_string();
    }

    // Try to find a good break point
    let truncated = &text[..max_chars];
    if let Some(last_space) = truncated.rfind(' ') {
        truncated[..last_space].to_string()
    } else {
        truncated.to_string()
    }
}

#[async_trait]
impl Tool for WebFetchTool {
    fn name(&self) -> &str {
        "web_fetch"
    }

    fn description(&self) -> &str {
        "Fetch and parse a web page, extracting text content. Returns the page title, main content, and optionally links."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to fetch and parse"
                },
                "extract_main": {
                    "type": "boolean",
                    "description": "Extract only main content (article/main) instead of all text (default: true)"
                },
                "include_links": {
                    "type": "boolean",
                    "description": "Include extracted links in the output (default: false)"
                },
                "max_chars": {
                    "type": "integer",
                    "description": "Maximum characters to return (default: 10000)",
                    "minimum": 1000,
                    "maximum": 50000
                }
            },
            "required": ["url"]
        })
    }

    async fn execute(&self, input: Value) -> Result<ToolResult> {
        let fetch_input: FetchInput = serde_json::from_value(input).map_err(|e| {
            cc_core::Error::ToolExecution(format!("Invalid input parameters: {}", e))
        })?;

        if fetch_input.url.trim().is_empty() {
            return Ok(ToolResult::error("URL cannot be empty"));
        }

        let max_chars = fetch_input.max_chars.clamp(1000, 50000);

        match self
            .fetch_url(
                &fetch_input.url,
                fetch_input.extract_main,
                fetch_input.include_links,
                max_chars,
            )
            .await
        {
            Ok(content) => Ok(ToolResult::success(content)),
            Err(e) => Ok(ToolResult::error(format!("Failed to fetch URL: {}", e))),
        }
    }
}

impl Default for WebFetchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_title() {
        let html = r#"<html><head><title>Test Page</title></head><body>Content</body></html>"#;
        let doc = Html::parse_document(html);
        assert_eq!(extract_title(&doc), "Test Page");
    }

    #[test]
    fn test_extract_title_empty() {
        let html = r#"<html><body>Content</body></html>"#;
        let doc = Html::parse_document(html);
        assert_eq!(extract_title(&doc), "No title");
    }

    #[test]
    fn test_extract_all_text() {
        let html = r#"<html><body><p>Hello World</p><p>Test Content</p></body></html>"#;
        let doc = Html::parse_document(html);
        let text = extract_all_text(&doc);
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
        assert!(text.contains("Test"));
        assert!(text.contains("Content"));
    }

    #[test]
    fn test_truncate_text() {
        let text = "This is a test sentence for truncation";
        let truncated = truncate_text(text, 20);
        assert!(truncated.len() <= 20);
        assert!(!truncated.ends_with("truncation"));
    }

    #[test]
    fn test_truncate_text_short() {
        let text = "Short text";
        let truncated = truncate_text(text, 100);
        assert_eq!(truncated, text);
    }

    #[test]
    fn test_fetch_input_parsing() {
        let input = json!({
            "url": "https://example.com",
            "include_links": true,
            "max_chars": 5000
        });

        let parsed: FetchInput = serde_json::from_value(input).unwrap();
        assert_eq!(parsed.url, "https://example.com");
        assert!(parsed.extract_main); // default
        assert!(parsed.include_links);
        assert_eq!(parsed.max_chars, 5000);
    }
}
