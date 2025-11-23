use std::fmt;

/// SameSite cookie attribute
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SameSite {
    /// Strict: Cookie only sent in first-party context
    Strict,
    /// Lax: Cookie sent on top-level navigation
    Lax,
    /// None: Cookie sent in all contexts (requires Secure)
    None,
}

impl fmt::Display for SameSite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SameSite::Strict => write!(f, "Strict"),
            SameSite::Lax => write!(f, "Lax"),
            SameSite::None => write!(f, "None"),
        }
    }
}

/// HTTP Cookie
/// 
/// # Example
/// 
/// ```
/// use firework::Cookie;
/// 
/// let cookie = Cookie::new("session_id", "abc123")
///     .http_only(true)
///     .secure(true)
///     .same_site(SameSite::Strict)
///     .max_age(3600)
///     .path("/");
/// ```
#[derive(Debug, Clone)]
pub struct Cookie {
    name: String,
    value: String,
    expires: Option<String>,
    max_age: Option<i64>,
    domain: Option<String>,
    path: Option<String>,
    secure: bool,
    http_only: bool,
    same_site: Option<SameSite>,
}

impl Cookie {
    /// Create a new cookie with name and value
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            expires: None,
            max_age: None,
            domain: None,
            path: Some("/".to_string()),
            secure: false,
            http_only: false,
            same_site: None,
        }
    }
    
    /// Get cookie name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get cookie value
    pub fn value(&self) -> &str {
        &self.value
    }
    
    /// Set expiration date (HTTP date format)
    pub fn expires(mut self, date: impl Into<String>) -> Self {
        self.expires = Some(date.into());
        self
    }
    
    /// Set max-age in seconds
    pub fn max_age(mut self, seconds: i64) -> Self {
        self.max_age = Some(seconds);
        self
    }
    
    /// Set domain
    pub fn domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }
    
    /// Set path
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }
    
    /// Set secure flag (HTTPS only)
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }
    
    /// Set HTTP-only flag (not accessible via JavaScript)
    pub fn http_only(mut self, http_only: bool) -> Self {
        self.http_only = http_only;
        self
    }
    
    /// Set SameSite attribute
    pub fn same_site(mut self, same_site: SameSite) -> Self {
        self.same_site = Some(same_site);
        self
    }
    
    /// Convert cookie to Set-Cookie header value
    pub fn to_header_value(&self) -> String {
        let mut parts = vec![format!("{}={}", self.name, self.value)];
        
        if let Some(ref expires) = self.expires {
            parts.push(format!("Expires={}", expires));
        }
        
        if let Some(max_age) = self.max_age {
            parts.push(format!("Max-Age={}", max_age));
        }
        
        if let Some(ref domain) = self.domain {
            parts.push(format!("Domain={}", domain));
        }
        
        if let Some(ref path) = self.path {
            parts.push(format!("Path={}", path));
        }
        
        if self.secure {
            parts.push("Secure".to_string());
        }
        
        if self.http_only {
            parts.push("HttpOnly".to_string());
        }
        
        if let Some(same_site) = self.same_site {
            parts.push(format!("SameSite={}", same_site));
        }
        
        parts.join("; ")
    }
    
    /// Parse a cookie from Cookie header value
    pub fn parse(s: &str) -> Option<Self> {
        let mut parts = s.split(';').map(|s| s.trim());
        
        // First part is name=value
        let first = parts.next()?;
        let (name, value) = first.split_once('=')?;
        
        let mut cookie = Cookie::new(name, value);
        
        // Parse attributes
        for part in parts {
            if part.eq_ignore_ascii_case("Secure") {
                cookie.secure = true;
            } else if part.eq_ignore_ascii_case("HttpOnly") {
                cookie.http_only = true;
            } else if let Some((key, val)) = part.split_once('=') {
                let key = key.trim();
                let val = val.trim();
                
                if key.eq_ignore_ascii_case("Expires") {
                    cookie.expires = Some(val.to_string());
                } else if key.eq_ignore_ascii_case("Max-Age") {
                    cookie.max_age = val.parse().ok();
                } else if key.eq_ignore_ascii_case("Domain") {
                    cookie.domain = Some(val.to_string());
                } else if key.eq_ignore_ascii_case("Path") {
                    cookie.path = Some(val.to_string());
                } else if key.eq_ignore_ascii_case("SameSite") {
                    cookie.same_site = match val {
                        "Strict" => Some(SameSite::Strict),
                        "Lax" => Some(SameSite::Lax),
                        "None" => Some(SameSite::None),
                        _ => None,
                    };
                }
            }
        }
        
        Some(cookie)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cookie_creation() {
        let cookie = Cookie::new("session", "abc123");
        assert_eq!(cookie.name(), "session");
        assert_eq!(cookie.value(), "abc123");
    }
    
    #[test]
    fn test_cookie_builder() {
        let cookie = Cookie::new("session", "abc123")
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Strict)
            .max_age(3600)
            .path("/api");
        
        assert!(cookie.http_only);
        assert!(cookie.secure);
        assert_eq!(cookie.same_site, Some(SameSite::Strict));
        assert_eq!(cookie.max_age, Some(3600));
        assert_eq!(cookie.path, Some("/api".to_string()));
    }
    
    #[test]
    fn test_cookie_to_header() {
        let cookie = Cookie::new("session", "abc123")
            .http_only(true)
            .secure(true)
            .max_age(3600);
        
        let header = cookie.to_header_value();
        assert!(header.contains("session=abc123"));
        assert!(header.contains("HttpOnly"));
        assert!(header.contains("Secure"));
        assert!(header.contains("Max-Age=3600"));
    }
    
    #[test]
    fn test_cookie_parse() {
        let cookie = Cookie::parse("session=abc123; HttpOnly; Secure; Max-Age=3600").unwrap();
        assert_eq!(cookie.name(), "session");
        assert_eq!(cookie.value(), "abc123");
        assert!(cookie.http_only);
        assert!(cookie.secure);
        assert_eq!(cookie.max_age, Some(3600));
    }
    
    #[test]
    fn test_samesite() {
        let cookie = Cookie::new("test", "value").same_site(SameSite::Lax);
        let header = cookie.to_header_value();
        assert!(header.contains("SameSite=Lax"));
    }
}
