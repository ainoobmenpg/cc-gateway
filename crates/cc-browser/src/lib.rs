//! cc-browser: Browser automation tools for cc-gateway
//!
//! This crate provides browser automation capabilities using headless Chrome.

pub mod error;
pub mod tools;

pub use error::{BrowserError, Result};
pub use tools::{
    BrowserClickTool, BrowserExtractTool, BrowserNavigateTool, BrowserScreenshotTool,
    BrowserTypeTool,
};
