//! cc-browser: Browser automation tools for cc-gateway
//!
//! This crate provides browser automation capabilities using headless Chrome.
//!
//! ## Features
//!
//! - Headless Chrome automation via headless_chrome crate
//! - Screenshot capture (full page or viewport)
//! - Form input and element interaction
//! - Text extraction and JavaScript execution
//! - Session management with configurable timeouts
//!
//! ## Usage
//!
//! ```rust,ignore
//! use cc_browser::{BrowserManager, BrowserNavigateTool};
//! use cc_core::ToolManager;
//! use std::sync::Arc;
//!
//! // Create a tool manager
//! let mut manager = ToolManager::new();
//!
//! // Register browser tools
//! cc_browser::register_browser_tools(&mut manager);
//!
//! // Or create tools with custom configuration
//! let browser_manager = BrowserManager::new();
//! manager.register(Arc::new(BrowserNavigateTool::new(browser_manager)));
//! ```

pub mod error;
pub mod session;
pub mod tools;

pub use error::{BrowserError, Result};
pub use session::{BrowserConfig, BrowserConfigBuilder, BrowserSession};
pub use tools::{
    BrowserClickTool, BrowserEvaluateTool, BrowserExtractTool, BrowserManager,
    BrowserNavigateTool, BrowserScreenshotTool, BrowserTypeTool, BrowserWaitTool,
};
