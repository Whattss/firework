use crate::{FromRequest, Request, Response, Error, Result, Json, Query, Path};
use serde::de::DeserializeOwned;
use validator::{Validate, ValidationErrors};
use std::fmt;

/// Validated extractor - automatically validates request data
/// 
/// # Example
/// 
/// ```rust
/// use firework::{Validated, Json};
/// use validator::Validate;
/// use serde::Deserialize;
/// 
/// #[derive(Deserialize, Validate)]
/// struct CreateUser {
///     #[validate(email)]
///     email: String,
///     
///     #[validate(length(min = 8, max = 128))]
///     password: String,
///     
///     #[validate(range(min = 18, max = 120))]
///     age: u8,
/// }
/// 
/// #[post("/users")]
/// async fn create_user(Validated(Json(user)): Validated<Json<CreateUser>>) -> Response {
///     // user is already validated
///     json!({"created": true})
/// }
/// ```
pub struct Validated<T>(pub T);

impl<T> Validated<T> {
    /// Extract the inner value
    pub fn into_inner(self) -> T {
        self.0
    }
}

/// Validation error with detailed messages
#[derive(Debug)]
pub struct ValidationError {
    pub errors: ValidationErrors,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation failed: {}", self.errors)
    }
}

impl std::error::Error for ValidationError {}

impl From<ValidationErrors> for ValidationError {
    fn from(errors: ValidationErrors) -> Self {
        Self { errors }
    }
}

// Validated<Json<T>>
#[async_trait::async_trait]
impl<T> FromRequest for Validated<Json<T>>
where
    T: DeserializeOwned + Validate + Send,
{
    async fn from_request(req: &mut Request, res: &mut Response) -> Result<Self> {
        // First extract JSON
        let Json(data) = Json::<T>::from_request(req, res).await?;
        
        // Then validate
        data.validate()
            .map_err(|e| Error::BadRequest(format!("Validation failed: {}", format_validation_errors(&e))))?;
        
        Ok(Validated(Json(data)))
    }
}

// Validated<Query<T>>
#[async_trait::async_trait]
impl<T> FromRequest for Validated<Query<T>>
where
    T: DeserializeOwned + Validate + Send,
{
    async fn from_request(req: &mut Request, res: &mut Response) -> Result<Self> {
        let Query(data) = Query::<T>::from_request(req, res).await?;
        
        data.validate()
            .map_err(|e| Error::BadRequest(format!("Query validation failed: {}", format_validation_errors(&e))))?;
        
        Ok(Validated(Query(data)))
    }
}

/// Format validation errors into readable message
fn format_validation_errors(errors: &ValidationErrors) -> String {
    let mut messages = Vec::new();
    
    for (field, field_errors) in errors.field_errors() {
        for error in field_errors {
            let msg = error.message
                .as_ref()
                .map(|m| m.to_string())
                .unwrap_or_else(|| {
                    match error.code.as_ref() {
                        "email" => format!("{} must be a valid email", field),
                        "url" => format!("{} must be a valid URL", field),
                        "length" => format!("{} length is invalid", field),
                        "range" => format!("{} is out of range", field),
                        "must_match" => format!("{} does not match", field),
                        "required" => format!("{} is required", field),
                        _ => format!("{} is invalid", field),
                    }
                });
            messages.push(msg);
        }
    }
    
    messages.join(", ")
}

/// Custom validators module
pub mod validators {
    use validator::ValidationError;
    
    /// Validate username (alphanumeric + underscore, 3-20 chars)
    pub fn validate_username(username: &str) -> Result<(), ValidationError> {
        if username.len() < 3 {
            return Err(ValidationError::new("username_too_short"));
        }
        if username.len() > 20 {
            return Err(ValidationError::new("username_too_long"));
        }
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(ValidationError::new("username_invalid_chars"));
        }
        Ok(())
    }
    
    /// Validate password strength
    pub fn validate_strong_password(password: &str) -> Result<(), ValidationError> {
        if password.len() < 8 {
            return Err(ValidationError::new("password_too_short"));
        }
        
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_digit = password.chars().any(|c| c.is_numeric());
        
        if !has_lowercase || !has_uppercase || !has_digit {
            return Err(ValidationError::new("password_too_weak"));
        }
        
        Ok(())
    }
    
    /// Validate phone number (simple validation)
    pub fn validate_phone(phone: &str) -> Result<(), ValidationError> {
        let digits: String = phone.chars().filter(|c| c.is_numeric()).collect();
        
        if digits.len() < 10 || digits.len() > 15 {
            return Err(ValidationError::new("phone_invalid"));
        }
        
        Ok(())
    }
    
    /// Validate slug (URL-friendly string)
    pub fn validate_slug(slug: &str) -> Result<(), ValidationError> {
        if slug.is_empty() {
            return Err(ValidationError::new("slug_empty"));
        }
        
        if !slug.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(ValidationError::new("slug_invalid_chars"));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    
    #[derive(Deserialize, Validate)]
    struct TestData {
        #[validate(email)]
        email: String,
        
        #[validate(length(min = 3, max = 20))]
        name: String,
        
        #[validate(range(min = 18, max = 120))]
        age: u8,
    }
    
    #[test]
    fn test_valid_data() {
        let data = TestData {
            email: "user@example.com".to_string(),
            name: "John".to_string(),
            age: 25,
        };
        
        assert!(data.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_email() {
        let data = TestData {
            email: "invalid-email".to_string(),
            name: "John".to_string(),
            age: 25,
        };
        
        assert!(data.validate().is_err());
    }
    
    #[test]
    fn test_invalid_length() {
        let data = TestData {
            email: "user@example.com".to_string(),
            name: "AB".to_string(), // Too short
            age: 25,
        };
        
        assert!(data.validate().is_err());
    }
    
    #[test]
    fn test_invalid_range() {
        let data = TestData {
            email: "user@example.com".to_string(),
            name: "John".to_string(),
            age: 150, // Out of range
        };
        
        assert!(data.validate().is_err());
    }
    
    #[test]
    fn test_username_validator() {
        assert!(validators::validate_username("john_doe").is_ok());
        assert!(validators::validate_username("ab").is_err()); // Too short
        assert!(validators::validate_username("john-doe").is_err()); // Invalid char
    }
    
    #[test]
    fn test_strong_password() {
        assert!(validators::validate_strong_password("Password123").is_ok());
        assert!(validators::validate_strong_password("weak").is_err());
        assert!(validators::validate_strong_password("lowercase123").is_err());
    }
    
    #[test]
    fn test_phone_validator() {
        assert!(validators::validate_phone("1234567890").is_ok());
        assert!(validators::validate_phone("+1 (234) 567-8900").is_ok());
        assert!(validators::validate_phone("123").is_err()); // Too short
    }
    
    #[test]
    fn test_slug_validator() {
        assert!(validators::validate_slug("my-post").is_ok());
        assert!(validators::validate_slug("my_post_123").is_ok());
        assert!(validators::validate_slug("my post").is_err()); // Space not allowed
    }
}
