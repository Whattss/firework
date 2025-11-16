# 📤 File Uploads

Handle file uploads in Firework.

---

## Basic Upload

```rust
use std::io::Write;

#[post("/upload")]
async fn upload(req: Request) -> Result<Json<serde_json::Value>, Error> {
    let body = req.body;
    
    // Validate size
    if body.len() > 10_000_000 {
        return Err(Error::PayloadTooLarge("File too large (max 10MB)".into()));
    }
    
    // Generate filename
    let filename = format!("upload_{}.dat", uuid::Uuid::new_v4());
    let filepath = format!("./uploads/{}", filename);
    
    // Save file
    let mut file = std::fs::File::create(&filepath)
        .map_err(|e| Error::Internal(e.to_string()))?;
    
    file.write_all(&body)
        .map_err(|e| Error::Internal(e.to_string()))?;
    
    Ok(Json(serde_json::json!({
        "filename": filename,
        "size": body.len(),
        "path": filepath
    })))
}
```

---

## Test Upload

```bash
curl -X POST http://localhost:8080/upload \
  -H "Content-Type: application/octet-stream" \
  --data-binary @myfile.jpg
```
