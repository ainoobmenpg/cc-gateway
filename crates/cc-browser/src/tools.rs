//! Browser automation tools
//!
//! Provides Tool implementations for browser automation using headless Chrome.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

use cc_core::{Tool, ToolResult};

use crate::error::{BrowserError, Result};
use crate::session::{BrowserConfig, BrowserSession};

/// Shared browser session manager
///
/// Maintains a single browser instance that can be reused across tool calls.
pub struct BrowserManager {
    session: Arc<Mutex<Option<BrowserSession>>>,
    config: BrowserConfig,
}

impl BrowserManager {
    /// Create a new browser manager with default configuration
    pub fn new() -> Self {
        Self::with_config(BrowserConfig::default())
    }

    /// Create a new browser manager with custom configuration
    pub fn with_config(config: BrowserConfig) -> Self {
        Self {
            session: Arc::new(Mutex::new(None)),
            config,
        }
    }

    /// Get or create a browser session
    pub async fn get_session(&self) -> Result<BrowserSession> {
        let mut session_guard = self.session.lock().await;

        if session_guard.is_none() {
            info!("Creating new browser session");
            let new_session = BrowserSession::with_config(self.config.clone())?;
            *session_guard = Some(new_session);
        }

        // Clone is not directly possible for BrowserSession, so we return a reference
        // In practice, we use the session directly through the guard
        Err(BrowserError::Initialization(
            "Use execute_with_session for operations".to_string(),
        ))
    }

    /// Execute an operation with the browser session
    pub async fn execute_with_session<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&BrowserSession) -> Result<T>,
    {
        let mut session_guard = self.session.lock().await;

        if session_guard.is_none() {
            info!("Creating new browser session");
            let new_session = BrowserSession::with_config(self.config.clone())?;
            *session_guard = Some(new_session);
        }

        let session = session_guard.as_ref().ok_or_else(|| {
            BrowserError::Initialization("Failed to get browser session".to_string())
        })?;

        f(session)
    }

    /// Close the browser session
    pub async fn close(&self) {
        let mut session_guard = self.session.lock().await;
        if session_guard.take().is_some() {
            info!("Browser session closed");
        }
    }

    /// Check if a session is active
    pub async fn is_active(&self) -> bool {
        self.session.lock().await.is_some()
    }
}

impl Default for BrowserManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for BrowserManager {
    fn clone(&self) -> Self {
        Self {
            session: self.session.clone(),
            config: self.config.clone(),
        }
    }
}

/// Browser navigate tool
pub struct BrowserNavigateTool {
    manager: BrowserManager,
}

impl BrowserNavigateTool {
    /// Create a new navigate tool with shared browser manager
    pub fn new(manager: BrowserManager) -> Self {
        Self { manager }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(BrowserManager::new())
    }
}

#[async_trait]
impl Tool for BrowserNavigateTool {
    fn name(&self) -> &str {
        "browser_navigate"
    }

    fn description(&self) -> &str {
        "Navigate to a URL in a headless browser and return the page title"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to navigate to"
                }
            },
            "required": ["url"]
        })
    }

    async fn execute(&self, input: Value) -> cc_core::Result<ToolResult> {
        let url = input["url"]
            .as_str()
            .ok_or_else(|| cc_core::Error::ToolExecution("Missing url parameter".to_string()))?;

        debug!("browser_navigate: {}", url);

        let result = self
            .manager
            .execute_with_session(|session| {
                let title = session.navigate(url)?;
                Ok(json!({
                    "url": url,
                    "title": title,
                    "status": "success"
                }))
            })
            .await
            .map_err(|e| cc_core::Error::ToolExecution(e.to_string()))?;

        Ok(ToolResult::success(
            serde_json::to_string(&result).unwrap_or_default(),
        ))
    }
}

/// Browser click tool
pub struct BrowserClickTool {
    manager: BrowserManager,
}

impl BrowserClickTool {
    pub fn new(manager: BrowserManager) -> Self {
        Self { manager }
    }

    pub fn with_defaults() -> Self {
        Self::new(BrowserManager::new())
    }
}

#[async_trait]
impl Tool for BrowserClickTool {
    fn name(&self) -> &str {
        "browser_click"
    }

    fn description(&self) -> &str {
        "Click an element in the browser using CSS selector"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "selector": {
                    "type": "string",
                    "description": "CSS selector for the element to click"
                },
                "wait": {
                    "type": "boolean",
                    "description": "Wait for navigation after click (default: true)",
                    "default": true
                }
            },
            "required": ["selector"]
        })
    }

    async fn execute(&self, input: Value) -> cc_core::Result<ToolResult> {
        let selector = input["selector"]
            .as_str()
            .ok_or_else(|| cc_core::Error::ToolExecution("Missing selector parameter".to_string()))?;

        debug!("browser_click: {}", selector);

        let result = self
            .manager
            .execute_with_session(|session| {
                session.click(selector)?;
                Ok(json!({
                    "selector": selector,
                    "action": "clicked",
                    "status": "success"
                }))
            })
            .await
            .map_err(|e| cc_core::Error::ToolExecution(e.to_string()))?;

        Ok(ToolResult::success(
            serde_json::to_string(&result).unwrap_or_default(),
        ))
    }
}

/// Browser type tool
pub struct BrowserTypeTool {
    manager: BrowserManager,
}

impl BrowserTypeTool {
    pub fn new(manager: BrowserManager) -> Self {
        Self { manager }
    }

    pub fn with_defaults() -> Self {
        Self::new(BrowserManager::new())
    }
}

#[async_trait]
impl Tool for BrowserTypeTool {
    fn name(&self) -> &str {
        "browser_type"
    }

    fn description(&self) -> &str {
        "Type text into an input field in the browser"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "selector": {
                    "type": "string",
                    "description": "CSS selector for the input field"
                },
                "text": {
                    "type": "string",
                    "description": "Text to type into the field"
                },
                "clear_first": {
                    "type": "boolean",
                    "description": "Clear the field before typing (default: true)",
                    "default": true
                }
            },
            "required": ["selector", "text"]
        })
    }

    async fn execute(&self, input: Value) -> cc_core::Result<ToolResult> {
        let selector = input["selector"]
            .as_str()
            .ok_or_else(|| cc_core::Error::ToolExecution("Missing selector parameter".to_string()))?;
        let text = input["text"]
            .as_str()
            .ok_or_else(|| cc_core::Error::ToolExecution("Missing text parameter".to_string()))?;

        debug!("browser_type: {} -> {}", selector, text);

        let result = self
            .manager
            .execute_with_session(|session| {
                session.type_text(selector, text)?;
                Ok(json!({
                    "selector": selector,
                    "text": text,
                    "action": "typed",
                    "status": "success"
                }))
            })
            .await
            .map_err(|e| cc_core::Error::ToolExecution(e.to_string()))?;

        Ok(ToolResult::success(
            serde_json::to_string(&result).unwrap_or_default(),
        ))
    }
}

/// Browser screenshot tool
pub struct BrowserScreenshotTool {
    manager: BrowserManager,
}

impl BrowserScreenshotTool {
    pub fn new(manager: BrowserManager) -> Self {
        Self { manager }
    }

    pub fn with_defaults() -> Self {
        Self::new(BrowserManager::new())
    }
}

#[async_trait]
impl Tool for BrowserScreenshotTool {
    fn name(&self) -> &str {
        "browser_screenshot"
    }

    fn description(&self) -> &str {
        "Take a screenshot of the current page and return as base64"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "full_page": {
                    "type": "boolean",
                    "description": "Whether to capture the full page (default: false)",
                    "default": false
                },
                "selector": {
                    "type": "string",
                    "description": "CSS selector for specific element screenshot (optional)"
                }
            }
        })
    }

    async fn execute(&self, input: Value) -> cc_core::Result<ToolResult> {
        let full_page = input["full_page"].as_bool().unwrap_or(false);
        let _selector = input["selector"].as_str(); // Element screenshot not yet supported

        debug!("browser_screenshot: full_page={}", full_page);

        let result = self
            .manager
            .execute_with_session(|session| {
                let screenshot_data = session.screenshot(full_page)?;
                let base64_data =
                    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &screenshot_data);

                Ok(json!({
                    "image": base64_data,
                    "format": "png",
                    "size_bytes": screenshot_data.len(),
                    "full_page": full_page,
                    "status": "success"
                }))
            })
            .await
            .map_err(|e| cc_core::Error::ToolExecution(e.to_string()))?;

        Ok(ToolResult::success(
            serde_json::to_string(&result).unwrap_or_default(),
        ))
    }
}

/// Browser extract tool
pub struct BrowserExtractTool {
    manager: BrowserManager,
}

impl BrowserExtractTool {
    pub fn new(manager: BrowserManager) -> Self {
        Self { manager }
    }

    pub fn with_defaults() -> Self {
        Self::new(BrowserManager::new())
    }
}

#[async_trait]
impl Tool for BrowserExtractTool {
    fn name(&self) -> &str {
        "browser_extract"
    }

    fn description(&self) -> &str {
        "Extract text content from the current page or specific element"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "selector": {
                    "type": "string",
                    "description": "CSS selector to extract text from (optional, defaults to body)"
                },
                "include_html": {
                    "type": "boolean",
                    "description": "Include HTML source instead of just text (default: false)",
                    "default": false
                }
            }
        })
    }

    async fn execute(&self, input: Value) -> cc_core::Result<ToolResult> {
        let selector = input["selector"].as_str().unwrap_or("body");
        let include_html = input["include_html"].as_bool().unwrap_or(false);

        debug!("browser_extract: selector={}", selector);

        let result = self
            .manager
            .execute_with_session(|session| {
                if include_html {
                    let html = session.page_source()?;
                    Ok(json!({
                        "selector": selector,
                        "html": html,
                        "type": "html",
                        "status": "success"
                    }))
                } else {
                    let text = session.extract_text(selector)?;
                    Ok(json!({
                        "selector": selector,
                        "text": text,
                        "type": "text",
                        "status": "success"
                    }))
                }
            })
            .await
            .map_err(|e| cc_core::Error::ToolExecution(e.to_string()))?;

        Ok(ToolResult::success(
            serde_json::to_string(&result).unwrap_or_default(),
        ))
    }
}

/// Browser evaluate tool (JavaScript execution)
pub struct BrowserEvaluateTool {
    manager: BrowserManager,
}

impl BrowserEvaluateTool {
    pub fn new(manager: BrowserManager) -> Self {
        Self { manager }
    }

    pub fn with_defaults() -> Self {
        Self::new(BrowserManager::new())
    }
}

#[async_trait]
impl Tool for BrowserEvaluateTool {
    fn name(&self) -> &str {
        "browser_evaluate"
    }

    fn description(&self) -> &str {
        "Execute JavaScript in the browser and return the result"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "script": {
                    "type": "string",
                    "description": "JavaScript code to execute"
                }
            },
            "required": ["script"]
        })
    }

    async fn execute(&self, input: Value) -> cc_core::Result<ToolResult> {
        let script = input["script"]
            .as_str()
            .ok_or_else(|| cc_core::Error::ToolExecution("Missing script parameter".to_string()))?;

        debug!("browser_evaluate: {} bytes", script.len());

        let result = self
            .manager
            .execute_with_session(|session| {
                let value = session.evaluate_js(script)?;
                Ok(json!({
                    "result": value,
                    "status": "success"
                }))
            })
            .await
            .map_err(|e| cc_core::Error::ToolExecution(e.to_string()))?;

        Ok(ToolResult::success(
            serde_json::to_string(&result).unwrap_or_default(),
        ))
    }
}

/// Browser wait tool
pub struct BrowserWaitTool {
    manager: BrowserManager,
}

impl BrowserWaitTool {
    pub fn new(manager: BrowserManager) -> Self {
        Self { manager }
    }

    pub fn with_defaults() -> Self {
        Self::new(BrowserManager::new())
    }
}

#[async_trait]
impl Tool for BrowserWaitTool {
    fn name(&self) -> &str {
        "browser_wait"
    }

    fn description(&self) -> &str {
        "Wait for an element to appear on the page"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "selector": {
                    "type": "string",
                    "description": "CSS selector for the element to wait for"
                },
                "timeout": {
                    "type": "integer",
                    "description": "Timeout in seconds (default: 10)",
                    "default": 10
                }
            },
            "required": ["selector"]
        })
    }

    async fn execute(&self, input: Value) -> cc_core::Result<ToolResult> {
        let selector = input["selector"]
            .as_str()
            .ok_or_else(|| cc_core::Error::ToolExecution("Missing selector parameter".to_string()))?;
        let timeout = input["timeout"].as_u64();

        debug!("browser_wait: {} (timeout: {:?})", selector, timeout);

        let result = self
            .manager
            .execute_with_session(|session| {
                session.wait_for(selector, timeout)?;
                Ok(json!({
                    "selector": selector,
                    "action": "waited",
                    "status": "success"
                }))
            })
            .await
            .map_err(|e| cc_core::Error::ToolExecution(e.to_string()))?;

        Ok(ToolResult::success(
            serde_json::to_string(&result).unwrap_or_default(),
        ))
    }
}

/// Register browser tools with a tool manager using a shared browser session
pub fn register_browser_tools(manager: &mut cc_core::ToolManager) {
    let browser_manager = BrowserManager::new();

    manager.register(Arc::new(BrowserNavigateTool::new(browser_manager.clone())));
    manager.register(Arc::new(BrowserClickTool::new(browser_manager.clone())));
    manager.register(Arc::new(BrowserTypeTool::new(browser_manager.clone())));
    manager.register(Arc::new(BrowserScreenshotTool::new(browser_manager.clone())));
    manager.register(Arc::new(BrowserExtractTool::new(browser_manager.clone())));
    manager.register(Arc::new(BrowserEvaluateTool::new(browser_manager.clone())));
    manager.register(Arc::new(BrowserWaitTool::new(browser_manager)));

    info!("Registered 7 browser automation tools");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_names() {
        let manager = BrowserManager::new();

        assert_eq!(BrowserNavigateTool::new(manager.clone()).name(), "browser_navigate");
        assert_eq!(BrowserClickTool::new(manager.clone()).name(), "browser_click");
        assert_eq!(BrowserTypeTool::new(manager.clone()).name(), "browser_type");
        assert_eq!(BrowserScreenshotTool::new(manager.clone()).name(), "browser_screenshot");
        assert_eq!(BrowserExtractTool::new(manager.clone()).name(), "browser_extract");
        assert_eq!(BrowserEvaluateTool::new(manager.clone()).name(), "browser_evaluate");
        assert_eq!(BrowserWaitTool::new(manager).name(), "browser_wait");
    }

    #[test]
    fn test_input_schemas() {
        let manager = BrowserManager::new();

        let navigate_schema = BrowserNavigateTool::new(manager.clone()).input_schema();
        assert!(navigate_schema["properties"]["url"].is_object());

        let click_schema = BrowserClickTool::new(manager.clone()).input_schema();
        assert!(click_schema["properties"]["selector"].is_object());

        let type_schema = BrowserTypeTool::new(manager.clone()).input_schema();
        assert!(type_schema["properties"]["selector"].is_object());
        assert!(type_schema["properties"]["text"].is_object());
    }

    #[tokio::test]
    async fn test_browser_manager_clone() {
        let manager1 = BrowserManager::new();
        let manager2 = manager1.clone();

        assert!(!manager1.is_active().await);
        assert!(!manager2.is_active().await);
    }
}
