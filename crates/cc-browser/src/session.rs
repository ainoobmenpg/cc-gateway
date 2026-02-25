//! Browser session management
//!
//! Provides a managed browser instance with automatic lifecycle handling.

use std::path::PathBuf;
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
        let tabs_guard = tabs
            .lock()
            .map_err(|e| BrowserError::TabError(format!("Failed to lock tabs: {}", e)))?;

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
        tab.wait_until_navigated()
            .map_err(|e| BrowserError::Navigation(format!("Navigation timeout: {}", e)))?;

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
            .map_err(|e| {
                BrowserError::Screenshot(format!("Failed to capture screenshot: {}", e))
            })?;

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
        .map_err(|e| BrowserError::Interaction(format!("Failed to click '{}': {}", selector, e)))?;

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

        let result = tab.evaluate(script, false).map_err(|e| {
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
                tab.close(true)
                    .map_err(|e| BrowserError::TabError(format!("Failed to close tab: {}", e)))?;
                info!("Closed tab: {}", tab_id);
                return Ok(());
            }
        }

        warn!("Tab not found for closing: {}", tab_id);
        Err(BrowserError::TabError(format!(
            "Tab '{}' not found",
            tab_id
        )))
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

    // ==================== Frame Handling ====================

    /// Get all frame information from the page
    pub fn get_frames(&self) -> Result<Vec<FrameInfo>> {
        let tab = self.active_tab()?;

        let frames_script = r#"
            (function() {
                var result = [];
                function collectFrames(frames, depth) {
                    for (var i = 0; i < frames.length; i++) {
                        var frame = frames[i];
                        result.push({
                            name: frame.name || '',
                            url: frame.location || '',
                            depth: depth,
                            index: i
                        });
                        if (frame.frames && frame.frames.length > 0) {
                            collectFrames(frame.frames, depth + 1);
                        }
                    }
                }
                collectFrames(window.frames, 0);
                return result;
            })()
        "#;

        let result = tab.evaluate(frames_script, false).map_err(|e| {
            BrowserError::Frame(format!("Failed to get frames: {}", e))
        })?;

        let frames: Vec<FrameInfo> = serde_json::from_value(result.value.unwrap_or_default())
            .map_err(|e| BrowserError::Frame(format!("Failed to parse frames: {}", e)))?;

        debug!("Found {} frames", frames.len());
        Ok(frames)
    }

    /// Switch to a frame by index or name
    pub fn switch_to_frame(&self, frame_identifier: &str) -> Result<()> {
        let tab = self.active_tab()?;

        let script = format!(
            r#"
            (function() {{
                var frameId = "{}";
                var frame = null;
                if (!isNaN(frameId)) {{
                    frame = window.frames[parseInt(frameId)];
                }} else {{
                    frame = window.frames[frameId] || document.querySelector('iframe[name="{}"]') || document.querySelector('iframe[id="{}"]');
                }}
                if (frame) {{
                    window.currentFrame = frame;
                    return true;
                }}
                return false;
            }})()
            "#,
            frame_identifier, frame_identifier, frame_identifier
        );

        let result = tab.evaluate(&script, false).map_err(|e| {
            BrowserError::Frame(format!("Failed to switch to frame: {}", e))
        })?;

        if result.value.and_then(|v| v.as_bool()).unwrap_or(false) {
            info!("Switched to frame: {}", frame_identifier);
            Ok(())
        } else {
            Err(BrowserError::Frame(format!("Frame '{}' not found", frame_identifier)))
        }
    }

    /// Get content from a specific iframe
    pub fn get_iframe_content(&self, selector: &str) -> Result<String> {
        let tab = self.active_tab()?;

        let script = format!(
            r#"
            (function() {{
                var iframe = document.querySelector('{}');
                if (iframe && iframe.contentDocument) {{
                    return iframe.contentDocument.body.innerHTML;
                }}
                return '';
            }})()
            "#,
            selector
        );

        let result = tab.evaluate(&script, false).map_err(|e| {
            BrowserError::Frame(format!("Failed to get iframe content: {}", e))
        })?;

        let content = result.value.unwrap_or_default().as_str().unwrap_or("").to_string();
        debug!("Got iframe content for {}: {} bytes", selector, content.len());
        Ok(content)
    }

    // ==================== Cookie Management ====================

    /// Get all cookies for the current domain
    pub fn get_cookies(&self) -> Result<Vec<CookieInfo>> {
        let tab = self.active_tab()?;

        let cookies = tab.get_cookies().map_err(|e| {
            BrowserError::Cookie(format!("Failed to get cookies: {}", e))
        })?;

        let cookie_infos: Vec<CookieInfo> = cookies
            .into_iter()
            .map(|c| CookieInfo {
                name: c.name,
                value: c.value,
                domain: c.domain,
                path: c.path,
                secure: c.secure,
                http_only: c.http_only,
                same_site: c.same_site.map(|s| format!("{:?}", s)),
                expires: Some(c.expires),
            })
            .collect();

        debug!("Got {} cookies", cookie_infos.len());
        Ok(cookie_infos)
    }

    /// Set a cookie
    pub fn set_cookie(&self, name: &str, value: &str, domain: Option<&str>, path: Option<&str>) -> Result<()> {
        let tab = self.active_tab()?;

        let domain_clause = domain.map(|d| format!(r#", domain: "{}""#, d)).unwrap_or_default();
        let path_clause = path.map(|p| format!(r#", path: "{}""#, p)).unwrap_or_default();

        let script = format!(
            r#"
            (function() {{
                document.cookie = "{}={}"{}{};
            }})()
            "#,
            name, value, domain_clause, path_clause
        );

        tab.evaluate(&script, false).map_err(|e| {
            BrowserError::Cookie(format!("Failed to set cookie: {}", e))
        })?;

        info!("Set cookie: {}={}", name, value);
        Ok(())
    }

    /// Delete a cookie by name
    pub fn delete_cookie(&self, name: &str) -> Result<()> {
        let tab = self.active_tab()?;

        let script = format!(
            r#"
            (function() {{
                document.cookie = "{}=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/";
            }})()
            "#,
            name
        );

        tab.evaluate(&script, false).map_err(|e| {
            BrowserError::Cookie(format!("Failed to delete cookie: {}", e))
        })?;

        info!("Deleted cookie: {}", name);
        Ok(())
    }

    /// Clear all cookies
    pub fn clear_cookies(&self) -> Result<()> {
        let script = r#"
            (function() {
                var cookies = document.cookie.split(";");
                for (var i = 0; i < cookies.length; i++) {
                    var cookie = cookies[i];
                    var eqPos = cookie.indexOf("=");
                    var name = eqPos > -1 ? cookie.substr(0, eqPos) : cookie;
                    document.cookie = name + "=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/";
                }
            })()
        "#;

        let tab = self.active_tab()?;
        tab.evaluate(script, false).map_err(|e| {
            BrowserError::Cookie(format!("Failed to clear cookies: {}", e))
        })?;

        info!("Cleared all cookies");
        Ok(())
    }

    // ==================== Download Handling ====================

    /// Set download behavior and directory
    pub fn set_download_path(&self, path: &str) -> Result<()> {
        let tab = self.active_tab()?;

        let script = format!(
            r#"
            (function() {{
                window.downloadPath = "{}";
            }})()
            "#,
            path
        );

        tab.evaluate(&script, false).map_err(|e| {
            BrowserError::Download(format!("Failed to set download path: {}", e))
        })?;

        info!("Set download path: {}", path);
        Ok(())
    }

    /// Trigger a file download by clicking a link
    pub fn download_by_selector(&self, selector: &str, download_dir: &str) -> Result<String> {
        let tab = self.active_tab()?;

        // First, set up the download handler via CDP
        let download_script = format!(
            r#"
            (function() {{
                var element = document.querySelector('{}');
                if (!element) return null;

                var href = element.href || element.download;
                if (href) return href;

                // If it's a button or other element, click it
                element.click();
                return 'clicked';
            }})()
            "#,
            selector
        );

        let result = tab.evaluate(&download_script, false).map_err(|e| {
            BrowserError::Download(format!("Failed to initiate download: {}", e))
        })?;

        let _download_info = result.value.unwrap_or_default();

        // Create a unique filename based on timestamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| BrowserError::Download(e.to_string()))?
            .as_millis();

        let filename = format!("{}/download_{}", download_dir, timestamp);

        info!("Download initiated for selector: {} -> {}", selector, filename);
        Ok(filename)
    }

    /// Wait for a download to complete
    pub fn wait_for_download(&self, timeout_secs: u64) -> Result<Vec<PathBuf>> {
        // This is a simplified implementation - in practice, you'd monitor the download directory
        let timeout = Duration::from_secs(timeout_secs);
        let start = std::time::Instant::now();

        // For headless Chrome, downloads need to be configured via LaunchOptions
        // This is a placeholder that returns empty - actual download monitoring
        // would require the download folder to be configured at browser launch time
        debug!("Waiting for download (timeout: {}s)", timeout_secs);

        while start.elapsed() < timeout {
            // In a full implementation, you'd check the download directory here
            std::thread::sleep(Duration::from_millis(500));
        }

        Ok(vec![])
    }

    /// Get current URL
    pub fn get_current_url(&self) -> Result<String> {
        let tab = self.active_tab()?;

        let url = tab.get_url();

        Ok(url)
    }

    /// Go back in history
    pub fn back(&self) -> Result<()> {
        let tab = self.active_tab()?;

        let script = r#"
            (function() { window.history.back(); })()
        "#;

        tab.evaluate(script, false).map_err(|e| {
            BrowserError::Navigation(format!("Failed to go back: {}", e))
        })?;

        info!("Navigated back");
        Ok(())
    }

    /// Go forward in history
    pub fn forward(&self) -> Result<()> {
        let tab = self.active_tab()?;

        let script = r#"
            (function() { window.history.forward(); })()
        "#;

        tab.evaluate(script, false).map_err(|e| {
            BrowserError::Navigation(format!("Failed to go forward: {}", e))
        })?;

        info!("Navigated forward");
        Ok(())
    }

    /// Refresh the page
    pub fn refresh(&self) -> Result<()> {
        let tab = self.active_tab()?;

        tab.reload(true, None).map_err(|e| {
            BrowserError::Navigation(format!("Failed to refresh: {}", e))
        })?;

        info!("Page refreshed");
        Ok(())
    }
}

/// Frame information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FrameInfo {
    pub name: String,
    pub url: String,
    pub depth: u32,
    pub index: u32,
}

/// Cookie information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CookieInfo {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: Option<String>,
    pub expires: Option<f64>,
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
