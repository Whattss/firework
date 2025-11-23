use crate::{FromRequest, Request, Response, Error, Result};
use bytes::Bytes;
use multer::Multipart;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// Uploaded file
#[derive(Debug, Clone)]
pub struct UploadedFile {
    /// Original filename
    pub filename: Option<String>,
    
    /// Content type (MIME type)
    pub content_type: Option<String>,
    
    /// File data
    pub data: Bytes,
    
    /// Size in bytes
    pub size: usize,
}

impl UploadedFile {
    /// Save file to disk
    pub async fn save(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let mut file = File::create(path).await?;
        file.write_all(&self.data).await?;
        file.flush().await?;
        Ok(())
    }
    
    /// Save file to disk with unique name
    pub async fn save_with_unique_name(&self, dir: impl AsRef<Path>) -> std::io::Result<PathBuf> {
        let filename = self.filename.as_deref().unwrap_or("upload");
        let unique_name = format!("{}_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0), filename);
        let path = dir.as_ref().join(unique_name);
        self.save(&path).await?;
        Ok(path)
    }
    
    /// Get file extension
    pub fn extension(&self) -> Option<&str> {
        self.filename
            .as_ref()
            .and_then(|f| Path::new(f).extension())
            .and_then(|e| e.to_str())
    }
    
    /// Check if file is an image (by MIME type)
    pub fn is_image(&self) -> bool {
        self.content_type
            .as_ref()
            .map(|ct| ct.starts_with("image/"))
            .unwrap_or(false)
    }
    
    /// Check if file size is within limit
    pub fn is_within_size(&self, max_bytes: usize) -> bool {
        self.size <= max_bytes
    }
}

/// Form data with files and fields
#[derive(Debug, Clone)]
pub struct FormData {
    /// Files uploaded
    pub files: HashMap<String, Vec<UploadedFile>>,
    
    /// Text fields
    pub fields: HashMap<String, Vec<String>>,
}

impl FormData {
    /// Get first file by field name
    pub fn file(&self, name: &str) -> Option<&UploadedFile> {
        self.files.get(name).and_then(|files| files.first())
    }
    
    /// Get all files for a field
    pub fn files_for(&self, name: &str) -> Option<&Vec<UploadedFile>> {
        self.files.get(name)
    }
    
    /// Get first field value by name
    pub fn field(&self, name: &str) -> Option<&str> {
        self.fields.get(name).and_then(|vals| vals.first()).map(|s| s.as_str())
    }
    
    /// Get all values for a field
    pub fn fields_for(&self, name: &str) -> Option<&Vec<String>> {
        self.fields.get(name)
    }
}

/// Multipart form data extractor
#[async_trait::async_trait]
impl FromRequest for FormData {
    async fn from_request(req: &mut Request, _res: &mut Response) -> Result<Self> {
        // Get boundary from Content-Type header
        let content_type = req.headers
            .get("content-type")
            .or_else(|| req.headers.get("Content-Type"))
            .and_then(|v| v.first())
            .ok_or_else(|| Error::BadRequest("Missing Content-Type header".into()))?;
        
        let boundary = multer::parse_boundary(content_type)
            .map_err(|e| Error::BadRequest(format!("Invalid multipart boundary: {}", e)))?;
        
        // Create stream from body bytes
        use futures_util::stream::{self, StreamExt};
        let body = std::mem::take(&mut req.body);
        let body_stream = stream::once(async move { Ok::<_, multer::Error>(Bytes::from(body)) });
        
        let mut multipart = Multipart::new(body_stream, boundary);
        
        let mut files: HashMap<String, Vec<UploadedFile>> = HashMap::new();
        let mut fields: HashMap<String, Vec<String>> = HashMap::new();
        
        // Parse fields
        while let Some(field) = multipart.next_field().await
            .map_err(|e| Error::BadRequest(format!("Multipart parse error: {}", e)))? 
        {
            let name = field.name().map(|s| s.to_string()).unwrap_or_else(|| "unknown".to_string());
            let filename = field.file_name().map(|s| s.to_string());
            let content_type = field.content_type().map(|m| m.to_string());
            
            let data = field.bytes().await
                .map_err(|e| Error::BadRequest(format!("Failed to read field data: {}", e)))?;
            
            if filename.is_some() {
                // It's a file
                let uploaded_file = UploadedFile {
                    filename,
                    content_type,
                    size: data.len(),
                    data,
                };
                
                files.entry(name).or_insert_with(Vec::new).push(uploaded_file);
            } else {
                // It's a text field
                let value = String::from_utf8_lossy(&data).to_string();
                fields.entry(name).or_insert_with(Vec::new).push(value);
            }
        }
        
        Ok(FormData { files, fields })
    }
}

/// File upload configuration
#[derive(Debug, Clone)]
pub struct UploadConfig {
    /// Maximum file size in bytes (default: 10MB)
    pub max_file_size: usize,
    
    /// Allowed file extensions (None = allow all)
    pub allowed_extensions: Option<Vec<String>>,
    
    /// Allowed MIME types (None = allow all)
    pub allowed_mime_types: Option<Vec<String>>,
    
    /// Upload directory
    pub upload_dir: PathBuf,
}

impl Default for UploadConfig {
    fn default() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024, // 10MB
            allowed_extensions: None,
            allowed_mime_types: None,
            upload_dir: PathBuf::from("uploads"),
        }
    }
}

impl UploadConfig {
    /// Validate an uploaded file against this config
    pub fn validate(&self, file: &UploadedFile) -> Result<()> {
        // Check size
        if file.size > self.max_file_size {
            return Err(Error::BadRequest(format!(
                "File too large: {} bytes (max: {})",
                file.size, self.max_file_size
            )));
        }
        
        // Check extension
        if let Some(ref allowed_exts) = self.allowed_extensions {
            if let Some(ext) = file.extension() {
                if !allowed_exts.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
                    return Err(Error::BadRequest(format!(
                        "File extension not allowed: {}",
                        ext
                    )));
                }
            } else {
                return Err(Error::BadRequest("File has no extension".into()));
            }
        }
        
        // Check MIME type
        if let Some(ref allowed_mimes) = self.allowed_mime_types {
            if let Some(ref content_type) = file.content_type {
                if !allowed_mimes.iter().any(|m| content_type.starts_with(m)) {
                    return Err(Error::BadRequest(format!(
                        "File type not allowed: {}",
                        content_type
                    )));
                }
            } else {
                return Err(Error::BadRequest("File has no content type".into()));
            }
        }
        
        Ok(())
    }
    
    /// Create upload directory if it doesn't exist
    pub async fn ensure_upload_dir(&self) -> std::io::Result<()> {
        tokio::fs::create_dir_all(&self.upload_dir).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_uploaded_file() {
        let file = UploadedFile {
            filename: Some("test.jpg".to_string()),
            content_type: Some("image/jpeg".to_string()),
            data: Bytes::from("fake image data"),
            size: 15,
        };
        
        assert_eq!(file.extension(), Some("jpg"));
        assert!(file.is_image());
        assert!(file.is_within_size(100));
        assert!(!file.is_within_size(10));
    }
    
    #[test]
    fn test_upload_config_validation() {
        let config = UploadConfig {
            max_file_size: 1024,
            allowed_extensions: Some(vec!["jpg".to_string(), "png".to_string()]),
            allowed_mime_types: Some(vec!["image/".to_string()]),
            upload_dir: PathBuf::from("uploads"),
        };
        
        let file = UploadedFile {
            filename: Some("test.jpg".to_string()),
            content_type: Some("image/jpeg".to_string()),
            data: Bytes::from("test"),
            size: 4,
        };
        
        assert!(config.validate(&file).is_ok());
        
        // Too large
        let large_file = UploadedFile {
            filename: Some("test.jpg".to_string()),
            content_type: Some("image/jpeg".to_string()),
            data: Bytes::from(vec![0u8; 2048]),
            size: 2048,
        };
        assert!(config.validate(&large_file).is_err());
        
        // Wrong extension
        let wrong_ext = UploadedFile {
            filename: Some("test.pdf".to_string()),
            content_type: Some("image/jpeg".to_string()),
            data: Bytes::from("test"),
            size: 4,
        };
        assert!(config.validate(&wrong_ext).is_err());
    }
    
    #[test]
    fn test_form_data() {
        let mut form = FormData {
            files: HashMap::new(),
            fields: HashMap::new(),
        };
        
        form.fields.insert("name".to_string(), vec!["John".to_string()]);
        form.fields.insert("tags".to_string(), vec!["tag1".to_string(), "tag2".to_string()]);
        
        assert_eq!(form.field("name"), Some("John"));
        assert_eq!(form.fields_for("tags").map(|v| v.len()), Some(2));
    }
}
