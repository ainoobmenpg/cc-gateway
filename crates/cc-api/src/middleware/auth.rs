//! Authentication middleware
//!
//! Provides API key authentication for protected endpoints.

use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};

/// Authentication extractor
pub struct Authenticated;

/// API key authentication middleware
pub async fn auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get API key from header
    let api_key = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| {
            value
                .strip_prefix("Bearer ")
                .map(|s| s.to_string())
        });

    // If no API key is configured, allow all requests
    // This is useful for development/testing
    // In production, you should always configure an API key
    let expected_key = std::env::var("API_KEY").ok();

    match (api_key, expected_key) {
        (Some(key), Some(expected)) if key == expected => {
            // Valid API key
            Ok(next.run(request).await)
        }
        (None, Some(_)) => {
            // API key required but not provided
            Err(StatusCode::UNAUTHORIZED)
        }
        (Some(_), Some(_)) => {
            // Invalid API key
            Err(StatusCode::UNAUTHORIZED)
        }
        (_, None) => {
            // No API key configured, allow request
            Ok(next.run(request).await)
        }
    }
}

/// Simple API key validation (for use in handlers)
pub fn validate_api_key(provided: Option<&str>, expected: Option<&str>) -> bool {
    match (provided, expected) {
        (Some(p), Some(e)) => p == e,
        (_, None) => true, // No key configured, allow
        (None, Some(_)) => false, // Key required but not provided
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_api_key_no_key_configured() {
        assert!(validate_api_key(None, None));
        assert!(validate_api_key(Some("any"), None));
    }

    #[test]
    fn test_validate_api_key_with_key_configured() {
        assert!(!validate_api_key(None, Some("secret")));
        assert!(!validate_api_key(Some("wrong"), Some("secret")));
        assert!(validate_api_key(Some("secret"), Some("secret")));
    }
}
