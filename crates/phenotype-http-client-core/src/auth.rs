//! HTTP authentication helpers.

/// Authentication credentials.
#[derive(Debug, Clone)]
pub enum AuthCredentials {
    Bearer(String),
    ApiKey { header: String, value: String },
    Basic { username: String, password: String },
}

impl AuthCredentials {
    pub fn bearer(token: impl Into<String>) -> Self {
        Self::Bearer(token.into())
    }

    pub fn api_key(header: impl Into<String>, value: impl Into<String>) -> Self {
        Self::ApiKey {
            header: header.into(),
            value: value.into(),
        }
    }

    pub fn basic(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self::Basic {
            username: username.into(),
            password: password.into(),
        }
    }

    /// Return the header name and value for this credential.
    pub fn to_header(&self) -> (String, String) {
        match self {
            Self::Bearer(token) => (
                "Authorization".to_string(),
                format!("Bearer {token}"),
            ),
            Self::ApiKey { header, value } => (header.clone(), value.clone()),
            Self::Basic { username, password } => {
                use std::io::Write;
                let mut buf = Vec::new();
                write!(buf, "{username}:{password}").unwrap();
                let encoded = base64_encode(&buf);
                (
                    "Authorization".to_string(),
                    format!("Basic {encoded}"),
                )
            }
        }
    }
}

/// Auth middleware that holds credentials.
#[derive(Debug, Clone)]
pub struct AuthMiddleware {
    credentials: AuthCredentials,
}

impl AuthMiddleware {
    pub fn new(credentials: AuthCredentials) -> Self {
        Self { credentials }
    }

    /// Get the auth header pair.
    pub fn header(&self) -> (String, String) {
        self.credentials.to_header()
    }

    /// Apply this middleware to a header list.
    pub fn apply_to_headers(&self, headers: &mut Vec<(String, String)>) {
        let (name, value) = self.header();
        headers.push((name, value));
    }
}

/// Simple base64 encoding (no external dep).
fn base64_encode(input: &[u8]) -> String {
    const CHARS: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bearer_header() {
        let creds = AuthCredentials::bearer("tok123");
        let (name, value) = creds.to_header();
        assert_eq!(name, "Authorization");
        assert_eq!(value, "Bearer tok123");
    }

    #[test]
    fn bearer_header_with_string() {
        let token = "secret_token_very_long".to_string();
        let creds = AuthCredentials::bearer(token.clone());
        let (name, value) = creds.to_header();
        assert_eq!(name, "Authorization");
        assert_eq!(value, format!("Bearer {}", token));
    }

    #[test]
    fn api_key_header() {
        let creds = AuthCredentials::api_key("X-API-Key", "secret");
        let (name, value) = creds.to_header();
        assert_eq!(name, "X-API-Key");
        assert_eq!(value, "secret");
    }

    #[test]
    fn api_key_custom_header_name() {
        let creds = AuthCredentials::api_key("Authorization-Token", "my-secret-key-123");
        let (name, value) = creds.to_header();
        assert_eq!(name, "Authorization-Token");
        assert_eq!(value, "my-secret-key-123");
    }

    #[test]
    fn basic_header() {
        let creds = AuthCredentials::basic("user", "pass");
        let (name, value) = creds.to_header();
        assert_eq!(name, "Authorization");
        assert!(value.starts_with("Basic "));
        // Basic auth encoding: base64("user:pass") = "dXNlcjpwYXNz"
        assert_eq!(value, "Basic dXNlcjpwYXNz");
    }

    #[test]
    fn basic_header_with_special_chars() {
        let creds = AuthCredentials::basic("admin@example.com", "p@ss:word");
        let (name, value) = creds.to_header();
        assert_eq!(name, "Authorization");
        assert!(value.starts_with("Basic "));
    }

    #[test]
    fn base64_encode_hello() {
        assert_eq!(base64_encode(b"Hello"), "SGVsbG8=");
    }

    #[test]
    fn base64_encode_empty() {
        assert_eq!(base64_encode(b""), "");
    }

    #[test]
    fn base64_encode_single_byte() {
        assert_eq!(base64_encode(b"A"), "QQ==");
    }

    #[test]
    fn base64_encode_two_bytes() {
        assert_eq!(base64_encode(b"AB"), "QUI=");
    }

    #[test]
    fn base64_encode_three_bytes() {
        assert_eq!(base64_encode(b"ABC"), "QUJD");
    }

    #[test]
    fn auth_middleware_bearer() {
        let middleware = AuthMiddleware::new(AuthCredentials::bearer("token"));
        let (name, value) = middleware.header();
        assert_eq!(name, "Authorization");
        assert_eq!(value, "Bearer token");
    }

    #[test]
    fn auth_middleware_apply_to_headers() {
        let middleware = AuthMiddleware::new(AuthCredentials::bearer("token"));
        let mut headers = vec![("Content-Type".to_string(), "application/json".to_string())];
        middleware.apply_to_headers(&mut headers);
        assert_eq!(headers.len(), 2);
        assert_eq!(headers[1].0, "Authorization");
        assert_eq!(headers[1].1, "Bearer token");
    }

    #[test]
    fn auth_credentials_clone() {
        let creds = AuthCredentials::bearer("token");
        let creds2 = creds.clone();
        let (_, value1) = creds.to_header();
        let (_, value2) = creds2.to_header();
        assert_eq!(value1, value2);
    }

    #[test]
    fn auth_middleware_clone() {
        let middleware =
            AuthMiddleware::new(AuthCredentials::api_key("X-Key", "value"));
        let middleware2 = middleware.clone();
        let (name1, value1) = middleware.header();
        let (name2, value2) = middleware2.header();
        assert_eq!(name1, name2);
        assert_eq!(value1, value2);
    }
}
