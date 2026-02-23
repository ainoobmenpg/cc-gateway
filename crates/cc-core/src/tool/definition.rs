//! Tool definition helpers
//!
//! Re-exports ToolDefinition from llm module and provides
//! helper functions for creating tool schemas.

use serde_json::{json, Value as JsonValue};

/// Tool definition for Claude API
///
/// Re-exported from llm module for convenience.
pub use crate::llm::ToolDefinition;

/// Helper functions for creating tool schemas
pub struct SchemaBuilder;

impl SchemaBuilder {
    /// Create a simple object schema with properties
    ///
    /// # Arguments
    /// * `properties` - A list of tuples (name, type, required)
    ///
    /// # Example
    /// ```ignore
    /// let schema = SchemaBuilder::object_schema(vec![
    ///     ("query", "string", true),
    ///     ("limit", "integer", false),
    /// ]);
    /// ```
    pub fn object_schema(properties: Vec<(&str, &str, bool)>) -> JsonValue {
        let props: serde_json::Map<String, JsonValue> = properties
            .iter()
            .map(|(name, type_str, _)| {
                (name.to_string(), json!({"type": type_str, "description": ""}))
            })
            .collect();

        let required: Vec<&str> = properties
            .iter()
            .filter(|(_, _, required)| *required)
            .map(|(name, _, _)| *name)
            .collect();

        json!({
            "type": "object",
            "properties": props,
            "required": required
        })
    }

    /// Create an object schema with descriptions for properties
    ///
    /// # Arguments
    /// * `properties` - A list of tuples (name, type, description, required)
    pub fn object_schema_with_descriptions(
        properties: Vec<(&str, &str, &str, bool)>,
    ) -> JsonValue {
        let props: serde_json::Map<String, JsonValue> = properties
            .iter()
            .map(|(name, type_str, desc, _)| {
                (
                    name.to_string(),
                    json!({"type": type_str, "description": desc}),
                )
            })
            .collect();

        let required: Vec<&str> = properties
            .iter()
            .filter(|(_, _, _, required)| *required)
            .map(|(name, _, _, _)| *name)
            .collect();

        json!({
            "type": "object",
            "properties": props,
            "required": required
        })
    }

    /// Create a string enum schema
    ///
    /// # Arguments
    /// * `enum_values` - List of allowed string values
    pub fn string_enum(enum_values: Vec<&str>) -> JsonValue {
        json!({
            "type": "string",
            "enum": enum_values
        })
    }
}
