//! Skills System for Claude Code Gateway
//!
//! This module provides a way to define custom tools/skills through
//! configuration files. Skills can be defined in YAML or TOML format
//! and are dynamically loaded and registered with the ToolManager.

pub mod loader;
pub mod types;

pub use loader::SkillLoader;
pub use types::{Skill, SkillConfig, SkillHttpConfig, SkillPromptConfig};
