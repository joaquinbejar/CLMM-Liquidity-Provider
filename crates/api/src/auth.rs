//! Authentication module for JWT and API key validation.
//!
//! Provides authentication middleware and utilities for:
//! - JWT token validation
//! - API key authentication
//! - Role-based access control

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, warn};

/// JWT claims structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID).
    pub sub: String,
    /// Expiration time (Unix timestamp).
    pub exp: u64,
    /// Issued at time (Unix timestamp).
    pub iat: u64,
    /// User roles.
    #[serde(default)]
    pub roles: Vec<String>,
}

impl Claims {
    /// Creates new claims.
    pub fn new(sub: impl Into<String>, exp: u64, roles: Vec<String>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            sub: sub.into(),
            exp,
            iat: now,
            roles,
        }
    }

    /// Checks if the token is expired.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.exp < now
    }

    /// Checks if the user has a specific role.
    #[must_use]
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
}

/// User roles for access control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    /// Read-only access.
    ReadOnly,
    /// Can execute operations.
    Execute,
    /// Full administrative access.
    Admin,
}

impl Role {
    /// Converts role to string.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::Execute => "execute",
            Self::Admin => "admin",
        }
    }

    /// Parses role from string.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "read_only" | "readonly" => Some(Self::ReadOnly),
            "execute" => Some(Self::Execute),
            "admin" => Some(Self::Admin),
            _ => None,
        }
    }
}

/// Authentication configuration.
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// JWT secret key.
    pub jwt_secret: String,
    /// Valid API keys.
    pub api_keys: HashSet<String>,
    /// Whether authentication is required.
    pub require_auth: bool,
    /// Token expiration time in seconds.
    pub token_expiry_secs: u64,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "default-secret-change-in-production".to_string(),
            api_keys: HashSet::new(),
            require_auth: false,
            token_expiry_secs: 3600, // 1 hour
        }
    }
}

/// Authentication state shared across handlers.
#[derive(Clone)]
pub struct AuthState {
    config: Arc<AuthConfig>,
}

impl AuthState {
    /// Creates a new authentication state.
    pub fn new(config: AuthConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    /// Validates an API key.
    #[must_use]
    pub fn validate_api_key(&self, key: &str) -> bool {
        self.config.api_keys.contains(key)
    }

    /// Validates a JWT token.
    pub fn validate_jwt(&self, token: &str) -> Result<Claims, AuthError> {
        // Simple JWT validation (in production, use a proper JWT library)
        // This is a simplified implementation for demonstration
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(AuthError::InvalidToken);
        }

        // Decode payload (base64)
        let payload = parts[1];
        let decoded = base64_decode(payload).map_err(|_| AuthError::InvalidToken)?;
        let claims: Claims =
            serde_json::from_slice(&decoded).map_err(|_| AuthError::InvalidToken)?;

        if claims.is_expired() {
            return Err(AuthError::TokenExpired);
        }

        Ok(claims)
    }

    /// Creates a JWT token for a user.
    pub fn create_token(&self, user_id: &str, roles: Vec<String>) -> Result<String, AuthError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| AuthError::TokenCreationFailed)?
            .as_secs();

        let claims = Claims::new(user_id, now + self.config.token_expiry_secs, roles);

        // Simple JWT creation (in production, use proper signing)
        let header = base64_encode(b"{\"alg\":\"HS256\",\"typ\":\"JWT\"}");
        let payload = base64_encode(
            &serde_json::to_vec(&claims).map_err(|_| AuthError::TokenCreationFailed)?,
        );
        let signature = base64_encode(b"signature"); // Simplified

        Ok(format!("{}.{}.{}", header, payload, signature))
    }

    /// Checks if authentication is required.
    #[must_use]
    pub fn require_auth(&self) -> bool {
        self.config.require_auth
    }
}

/// Authentication errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum AuthError {
    /// Missing authentication header.
    #[error("Missing authentication")]
    MissingAuth,
    /// Invalid token format.
    #[error("Invalid token")]
    InvalidToken,
    /// Token has expired.
    #[error("Token expired")]
    TokenExpired,
    /// Invalid API key.
    #[error("Invalid API key")]
    InvalidApiKey,
    /// Insufficient permissions.
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    /// Token creation failed.
    #[error("Failed to create token")]
    TokenCreationFailed,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::MissingAuth => StatusCode::UNAUTHORIZED,
            Self::InvalidToken => StatusCode::UNAUTHORIZED,
            Self::TokenExpired => StatusCode::UNAUTHORIZED,
            Self::InvalidApiKey => StatusCode::UNAUTHORIZED,
            Self::InsufficientPermissions => StatusCode::FORBIDDEN,
            Self::TokenCreationFailed => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = serde_json::json!({
            "error": self.to_string(),
            "code": status.as_u16()
        });

        (status, axum::Json(body)).into_response()
    }
}

/// Extracts authentication from request headers.
pub fn extract_auth(headers: &HeaderMap) -> Option<AuthMethod> {
    // Check for Bearer token
    if let Some(auth_header) = headers.get("Authorization")
        && let Ok(auth_str) = auth_header.to_str()
        && let Some(token) = auth_str.strip_prefix("Bearer ")
    {
        return Some(AuthMethod::Bearer(token.to_string()));
    }

    // Check for API key
    if let Some(api_key) = headers.get("X-API-Key")
        && let Ok(key) = api_key.to_str()
    {
        return Some(AuthMethod::ApiKey(key.to_string()));
    }

    None
}

/// Authentication method.
#[derive(Debug, Clone)]
pub enum AuthMethod {
    /// Bearer token (JWT).
    Bearer(String),
    /// API key.
    ApiKey(String),
}

/// Authentication middleware.
pub async fn auth_middleware(
    headers: HeaderMap,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AuthError> {
    // Extract auth state from extensions if available
    // For now, we'll do a simple check

    match extract_auth(&headers) {
        Some(AuthMethod::Bearer(token)) => {
            debug!("Bearer token authentication");
            // Validate token (simplified)
            if token.is_empty() {
                warn!("Empty bearer token");
                return Err(AuthError::InvalidToken);
            }
        }
        Some(AuthMethod::ApiKey(key)) => {
            debug!("API key authentication");
            if key.is_empty() {
                warn!("Empty API key");
                return Err(AuthError::InvalidApiKey);
            }
        }
        None => {
            // Allow unauthenticated requests for now (can be configured)
            debug!("No authentication provided");
        }
    }

    Ok(next.run(request).await)
}

/// Requires a specific role.
pub async fn require_role(
    required_role: Role,
    headers: HeaderMap,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AuthError> {
    match extract_auth(&headers) {
        Some(AuthMethod::Bearer(token)) => {
            // Parse claims and check role
            let parts: Vec<&str> = token.split('.').collect();
            if parts.len() == 3
                && let Ok(decoded) = base64_decode(parts[1])
                && let Ok(claims) = serde_json::from_slice::<Claims>(&decoded)
                && claims.has_role(required_role.as_str())
            {
                return Ok(next.run(request).await);
            }
            Err(AuthError::InsufficientPermissions)
        }
        Some(AuthMethod::ApiKey(_)) => {
            // API keys have full access for now
            Ok(next.run(request).await)
        }
        None => Err(AuthError::MissingAuth),
    }
}

// Helper functions for base64 encoding/decoding

fn base64_encode(data: &[u8]) -> String {
    use std::io::Write;
    let mut buf = Vec::new();
    {
        let mut encoder = Base64Encoder::new(&mut buf);
        encoder.write_all(data).ok();
    }
    String::from_utf8(buf).unwrap_or_default()
}

fn base64_decode(data: &str) -> Result<Vec<u8>, ()> {
    // Simple base64 URL-safe decoding
    let data = data.replace('-', "+").replace('_', "/");
    let padding = (4 - data.len() % 4) % 4;
    let padded = format!("{}{}", data, "=".repeat(padding));

    base64_decode_standard(&padded)
}

fn base64_decode_standard(data: &str) -> Result<Vec<u8>, ()> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = Vec::new();
    let mut buffer = 0u32;
    let mut bits = 0;

    for c in data.bytes() {
        if c == b'=' {
            break;
        }
        let value = ALPHABET.iter().position(|&x| x == c).ok_or(())? as u32;
        buffer = (buffer << 6) | value;
        bits += 6;

        if bits >= 8 {
            bits -= 8;
            result.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }

    Ok(result)
}

/// Simple base64 encoder.
struct Base64Encoder<'a> {
    output: &'a mut Vec<u8>,
    buffer: u32,
    bits: u8,
}

impl<'a> Base64Encoder<'a> {
    fn new(output: &'a mut Vec<u8>) -> Self {
        Self {
            output,
            buffer: 0,
            bits: 0,
        }
    }
}

impl std::io::Write for Base64Encoder<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

        for &byte in buf {
            self.buffer = (self.buffer << 8) | byte as u32;
            self.bits += 8;

            while self.bits >= 6 {
                self.bits -= 6;
                let index = ((self.buffer >> self.bits) & 0x3F) as usize;
                self.output.push(ALPHABET[index]);
            }
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if self.bits > 0 {
            const ALPHABET: &[u8] =
                b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
            let index = ((self.buffer << (6 - self.bits)) & 0x3F) as usize;
            self.output.push(ALPHABET[index]);

            // Add padding
            let padding = (3 - (self.bits / 8 + 1) % 3) % 3;
            for _ in 0..padding {
                self.output.push(b'=');
            }
        }
        Ok(())
    }
}

impl Drop for Base64Encoder<'_> {
    fn drop(&mut self) {
        let _ = std::io::Write::flush(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claims_expiry() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let valid_claims = Claims::new("user1", now + 3600, vec!["admin".to_string()]);
        assert!(!valid_claims.is_expired());

        let expired_claims = Claims::new("user1", now - 1, vec![]);
        assert!(expired_claims.is_expired());
    }

    #[test]
    fn test_claims_roles() {
        let claims = Claims::new(
            "user1",
            u64::MAX,
            vec!["admin".to_string(), "execute".to_string()],
        );

        assert!(claims.has_role("admin"));
        assert!(claims.has_role("execute"));
        assert!(!claims.has_role("read_only"));
    }

    #[test]
    fn test_role_parsing() {
        assert_eq!(Role::from_str("admin"), Some(Role::Admin));
        assert_eq!(Role::from_str("execute"), Some(Role::Execute));
        assert_eq!(Role::from_str("read_only"), Some(Role::ReadOnly));
        assert_eq!(Role::from_str("readonly"), Some(Role::ReadOnly));
        assert_eq!(Role::from_str("unknown"), None);
    }

    #[test]
    fn test_base64_roundtrip() {
        let original = b"Hello, World!";
        let encoded = base64_encode(original);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }
}
