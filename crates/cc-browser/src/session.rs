//! Browser session management
//!
//! Provides a managed browser instance with automatic lifecycle handling.

use std::sync::Arc;
use std::time::Duration;

use headless_chrome::{Browser, LaunchOptionsBuilder, Tab, protocol::cdp::Page};
use tracing::{debug, info, warn};

use crate::error::{BrowserError, Result};

/// Browser session configuration
#[derive(Debug, Clone)]
pub struct BrowserConfig {
    /// Whether to run in headless mode
    pub headless: bool,
    /// Window width in pixels
    pub width: u32,
    /// Window height in pixels
    pub height: u32,
    /// Navigation timeout in seconds
    pub navigation_timeout: u64,
    /// Element wait timeout in seconds
    pub element_timeout: u64,
    /// Enable GPU acceleration
    pub enable_gpu: bool,
    /// Custom user agent
    pub user_agent: Option<String>,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            headless: true,
            width: 1920,
            height: 1080,
            navigation_timeout: 30,
            element_timeout: 10,
            enable_gpu: false,
            user_agent: None,
        }
    }
}

impl BrowserConfig {
    /// Create a new configuration builder
    pub fn builder() -> BrowserConfigBuilder {
        BrowserConfigBuilder::default()
    }

    /// Create a headless configuration
    pub fn headless() -> Self {
        Self {
            headless: true,
            ..Default::default()
        }
    }

    /// Create a visible browser configuration
    pub fn visible() -> Self {
        Self {
            headless: false,
            ..Default::default()
        }
    }
}

/// Builder for BrowserConfig
#[derive(Default)]
pub struct BrowserConfigBuilder {
    config: BrowserConfig,
}

impl BrowserConfigBuilder {
    pub fn headless(mut self, headless: bool) -> Self {
        self.config.headless = headless;
        self
    }

    pub fn window_size(mut self, width: u32, height: u32) -> Self {
        self.config.width = width;
        self.config.height = height;
        self
    }

    pub fn navigation_timeout(mut self, seconds: u64) -> Self {
        self.config.navigation_timeout = seconds;
        self
    }

    pub fn element_timeout(mut self, seconds: u64) -> Self {
        self.config.element_timeout = seconds;
        self
    }

    pub fn enable_gpu(mut self, enable: bool) -> Self {
        self.config.enable_gpu = enable;
        self
    }

    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.config.user_agent = Some(user_agent.into());
        self
    }

    pub fn build(self) -> BrowserConfig {
        self.config
    }
}

/// Managed browser session
pub struct BrowserSession {
    browser: Browser,
    config: BrowserConfig,
}

impl BrowserSession {
    /// Create a new browser session with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(BrowserConfig::default())
    }

    /// Create a new browser session with custom configuration
    pub fn with_config(config: BrowserConfig) -> Result<Self> {
        use std::ffi::OsStr;

        info!("Creating browser session (headless: {})", config.headless);

        let mut args: Vec<String> = vec![
            format!("--window-size={},{}", config.width, config.height),
            "--no-sandbox".to_string(),
            "--disable-setuid-sandbox".to_string(),
            "--disable-dev-shm-usage".to_string(),
        ];

        if !config.enable_gpu {
            args.push("--disable-gpu".to_string());
            args.push("--disable-software-rasterizer".to_string());
        }

        if let Some(ref ua) = config.user_agent {
            args.push(format!("--user-agent={}", ua));
        }

        let os_args: Vec<&OsStr> = args.iter().map(OsStr::new).collect();

        let launch_options = LaunchOptionsBuilder::default()
            .headless(config.headless)
            .args(os_args)
            .build()
            .map_err(|e| {
                BrowserError::Initialization(format!("Failed to build launch options: {}", e))
            })?;

        let browser = Browser::new(launch_options).map_err(|e| {
            BrowserError::Initialization(format!("Failed to launch browser: {}", e))
        })?;

        info!("Browser session created successfully");

        Ok(Self { browser, config })
    }

    /// Get the active tab
    pub fn active_tab(&self) -> Result<Arc<Tab>> {
        let tabs = self.browser.get_tabs();
        let tabs_guard = tabs.lock().map_err(|e| {
            BrowserError::TabError(format!("Failed to lock tabs: {}", e))
        })?;

        tabs_guard
            .first()
            .cloned()
            .ok_or_else(|| BrowserError::TabError("No active tab available".to_string()))
    }

    /// Navigate to a URL
    pub fn navigate(&self, url: &str) -> Result<String> {
        let tab = self.active_tab()?;

        info!("Navigating to: {}", url);

        tab.navigate_to(url).map_err(|e| {
            BrowserError::Navigation(format!("Failed to navigate to {}: {}", url, e))
        })?;

        // Wait for page to load
        tab.wait_until_navigated().map_err(|e| {
            BrowserError::Navigation(format!("Navigation timeout: {}", e))
        })?;

        // Get page title
        let title = tab.get_title().unwrap_or_else(|_| "Unknown".to_string());

        info!("Navigated to: {} (title: {})", url, title);

        Ok(title)
    }

    /// Take a screenshot
    pub fn screenshot(&self, _full_page: bool) -> Result<Vec<u8>> {
        let tab = self.active_tab()?;

        debug!("Taking screenshot");

        let screenshot = tab
            .capture_screenshot(
                Page::CaptureScreenshotFormatOption::Png,
                Some(100),
                None,
                true,
            )
            .map_err(|e| BrowserError::Screenshot(format!("Failed to capture screenshot: {}", e)))?;

        info!("Screenshot captured: {} bytes", screenshot.len());

        Ok(screenshot)
    }

    /// Click an element
    pub fn click(&self, selector: &str) -> Result<()> {
        let tab = self.active_tab()?;

        info!("Clicking element: {}", selector);

        tab.wait_for_element_with_custom_timeout(
            selector,
            Duration::from_secs(self.config.element_timeout),
        )
        .map_err(|e| {
            BrowserError::ElementNotFound(format!("Element '{}' not found: {}", selector, e))
        })?
        .click()
        .map_err(|e| {
            BrowserError::Interaction(format!("Failed to click '{}': {}", selector, e))
        })?;

        info!("Clicked element: {}", selector);

        Ok(())
    }

    /// Type text into an element
    pub fn type_text(&self, selector: &str, text: &str) -> Result<()> {
        let tab = self.active_tab()?;

        info!("Typing into element: {} ({} chars)", selector, text.len());

        let element = tab
            .wait_for_element_with_custom_timeout(
                selector,
                Duration::from_secs(self.config.element_timeout),
            )
            .map_err(|e| {
                BrowserError::ElementNotFound(format!("Element '{}' not found: {}", selector, e))
            })?;

        // Use send_character method or type into the element
        element.click().map_err(|e| {
            BrowserError::Interaction(format!("Failed to focus '{}': {}", selector, e))
        })?;

        // Type character by character
        for c in text.chars() {
            tab.press_key(&c.to_string()).map_err(|e| {
                BrowserError::Interaction(format!("Failed to type character: {}", e))
            })?;
        }

        info!("Typed text into element: {}", selector);

        Ok(())
    }

    /// Extract text content from an element
    pub fn extract_text(&self, selector: &str) -> Result<String> {
        let tab = self.active_tab()?;

        debug!("Extracting text from: {}", selector);

        let element = tab
            .wait_for_element_with_custom_timeout(
                selector,
                Duration::from_secs(self.config.element_timeout),
            )
            .map_err(|e| {
                BrowserError::ElementNotFound(format!("Element '{}' not found: {}", selector, e))
            })?;

        let text = element
            .get_inner_text()
            .map_err(|e| BrowserError::Extraction(format!("Failed to extract text: {}", e)))?;

        debug!("Extracted {} characters from: {}", text.len(), selector);

        Ok(text)
    }

    /// Execute JavaScript
    pub fn evaluate_js(&self, script: &str) -> Result<serde_json::Value> {
        let tab = self.active_tab()?;

        debug!(
            "Executing JavaScript: {}...",
            &script[..std::cmp::min(50, script.len())]
        );

        let result = tab
            .evaluate(script, false)
            .map_err(|e| {
                BrowserError::Interaction(format!("JavaScript execution failed: {}", e))
            })?;

        Ok(result.value.unwrap_or(serde_json::Value::Null))
    }

    /// Wait for an element to appear
    pub fn wait_for(&self, selector: &str, timeout_secs: Option<u64>) -> Result<()> {
        let tab = self.active_tab()?;
        let timeout = Duration::from_secs(timeout_secs.unwrap_or(self.config.element_timeout));

        debug!("Waiting for element: {} (timeout: {:?})", selector, timeout);

        tab.wait_for_element_with_custom_timeout(selector, timeout)
            .map_err(|e| {
                BrowserError::Timeout(format!(
                    "Element '{}' not found within timeout: {}",
                    selector, e
                ))
            })?;

        Ok(())
    }

    /// Get all tabs
    pub fn tabs(&self) -> Vec<Arc<Tab>> {
        let tabs = self.browser.get_tabs();
        match tabs.lock() {
            Ok(guard) => guard.clone(),
            Err(_) => vec![],
        }
    }

    /// Create a new tab
    pub fn new_tab(&self) -> Result<Arc<Tab>> {
        let tab = self
            .browser
            .new_tab()
            .map_err(|e| BrowserError::TabError(format!("Failed to create new tab: {}", e)))?;

        info!("Created new tab");

        Ok(tab)
    }

    /// Close a tab by target ID
    pub fn close_tab(&self, tab_id: &str) -> Result<()> {
        let tabs = self.tabs();
        for tab in tabs {
            if tab.get_target_id() == tab_id {
                tab.close(true).map_err(|e| {
                    BrowserError::TabError(format!("Failed to close tab: {}", e))
                })?;
                info!("Closed tab: {}", tab_id);
                return Ok(());
            }
        }

        warn!("Tab not found for closing: {}", tab_id);
        Err(BrowserError::TabError(format!("Tab '{}' not found", tab_id)))
    }

    /// Get page HTML source
    pub fn page_source(&self) -> Result<String> {
        let tab = self.active_tab()?;

        let source = tab
            .get_content()
            .map_err(|e| BrowserError::Extraction(format!("Failed to get page source: {}", e)))?;

        Ok(source)
    }

    /// Get the browser configuration
    pub fn config(&self) -> &BrowserConfig {
        &self.config
    }
}

impl Drop for BrowserSession {
    fn drop(&mut self) {
        info!("Closing browser session");
        // Browser will be automatically closed when dropped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_config_default() {
        let config = BrowserConfig::default();
        assert!(config.headless);
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
    }

    #[test]
    fn test_browser_config_builder() {
        let config = BrowserConfig::builder()
            .headless(false)
            .window_size(1280, 720)
            .navigation_timeout(60)
            .user_agent("Custom Agent")
            .build();

        assert!(!config.headless);
        assert_eq!(config.width, 1280);
        assert_eq!(config.height, 720);
        assert_eq!(config.navigation_timeout, 60);
        assert_eq!(config.user_agent, Some("Custom Agent".to_string()));
    }

    #[test]
    fn test_browser_config_presets() {
        let headless = BrowserConfig::headless();
        assert!(headless.headless);

        let visible = BrowserConfig::visible();
        assert!(!visible.headless);
    }
}
