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
            Self::Bearer(token) => ("Authorization".to_string(), format!("Bearer {token}")),
            Self::ApiKey { header, value } => (header.clone(), value.clone()),
            Self::Basic { username, password } => {
                use std::io::Write;
                let mut buf = Vec::new();
                write!(buf, "{username}:{password}").unwrap();
                let encoded = base64_encode(&buf);
                ("Authorization".to_string(), format!("Basic {encoded}"))
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
}

/// Simple base64 encoding (no external dep).
fn base64_encode(input: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
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
    fn api_key_header() {
        let creds = AuthCredentials::api_key("X-API-Key", "secret");
        let (name, value) = creds.to_header();
        assert_eq!(name, "X-API-Key");
        assert_eq!(value, "secret");
    }

    #[test]
    fn basic_header() {
        let creds = AuthCredentials::basic("user", "pass");
        let (name, value) = creds.to_header();
        assert_eq!(name, "Authorization");
        assert!(value.starts_with("Basic "));
    }

    #[test]
    fn base64_encode_hello() {
        assert_eq!(base64_encode(b"Hello"), "SGVsbG8=");
    }
}
