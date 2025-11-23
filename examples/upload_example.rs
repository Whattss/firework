use firework::prelude::*;
use std::path::PathBuf;

#[get("/")]
async fn index() -> Response {
    html!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>File Upload Example</title>
    <style>
        body { font-family: sans-serif; max-width: 800px; margin: 50px auto; }
        form { background: #f0f0f0; padding: 20px; margin: 20px 0; border-radius: 5px; }
        input, button { display: block; margin: 10px 0; padding: 8px; width: 100%; }
        button { background: #007bff; color: white; border: none; cursor: pointer; }
        .result { margin-top: 20px; }
        .success { color: green; }
        .error { color: red; }
    </style>
</head>
<body>
    <h1>🔥 Firework File Upload Example</h1>
    
    <h2>Single File Upload</h2>
    <form id="singleForm" enctype="multipart/form-data">
        <input type="text" name="title" placeholder="File title" required>
        <input type="file" name="file" required>
        <button type="submit">Upload File</button>
    </form>
    <div id="singleResult" class="result"></div>
    
    <h2>Multiple Files Upload</h2>
    <form id="multipleForm" enctype="multipart/form-data">
        <input type="text" name="description" placeholder="Description">
        <input type="file" name="files" multiple required>
        <button type="submit">Upload Files</button>
    </form>
    <div id="multipleResult" class="result"></div>
    
    <h2>Image Upload (with validation)</h2>
    <form id="imageForm" enctype="multipart/form-data">
        <input type="text" name="caption" placeholder="Image caption">
        <input type="file" name="image" accept="image/*" required>
        <button type="submit">Upload Image</button>
    </form>
    <div id="imageResult" class="result"></div>
    
    <script>
        async function handleUpload(formId, endpoint, resultId) {
            const form = document.getElementById(formId);
            const formData = new FormData(form);
            
            try {
                const res = await fetch(endpoint, {
                    method: 'POST',
                    body: formData
                });
                const result = await res.json();
                
                const div = document.getElementById(resultId);
                if (res.ok) {
                    div.className = 'result success';
                    div.textContent = '✅ ' + JSON.stringify(result, null, 2);
                } else {
                    div.className = 'result error';
                    div.textContent = '❌ ' + (result.error || 'Upload failed');
                }
            } catch (err) {
                document.getElementById(resultId).className = 'result error';
                document.getElementById(resultId).textContent = '❌ ' + err;
            }
        }
        
        document.getElementById('singleForm').addEventListener('submit', (e) => {
            e.preventDefault();
            handleUpload('singleForm', '/upload', 'singleResult');
        });
        
        document.getElementById('multipleForm').addEventListener('submit', (e) => {
            e.preventDefault();
            handleUpload('multipleForm', '/upload/multiple', 'multipleResult');
        });
        
        document.getElementById('imageForm').addEventListener('submit', (e) => {
            e.preventDefault();
            handleUpload('imageForm', '/upload/image', 'imageResult');
        });
    </script>
</body>
</html>
    "#)
}

#[post("/upload")]
async fn upload_single(form: FormData) -> Response {
    // Get the uploaded file
    let file = match form.file("file") {
        Some(f) => f,
        None => return json!({"error": "No file uploaded"}),
    };
    
    let title = form.field("title").unwrap_or("Untitled");
    
    // Save file
    let upload_dir = PathBuf::from("uploads");
    tokio::fs::create_dir_all(&upload_dir).await.ok();
    
    match file.save_with_unique_name(&upload_dir).await {
        Ok(path) => json!({
            "success": true,
            "title": title,
            "filename": file.filename,
            "size": file.size,
            "content_type": file.content_type,
            "path": path.to_string_lossy(),
        }),
        Err(e) => json!({"error": format!("Failed to save file: {}", e)}),
    }
}

#[post("/upload/multiple")]
async fn upload_multiple(form: FormData) -> Response {
    let description = form.field("description").unwrap_or("No description");
    
    // Get all uploaded files
    let files = match form.files_for("files") {
        Some(f) => f,
        None => return json!({"error": "No files uploaded"}),
    };
    
    let upload_dir = PathBuf::from("uploads");
    tokio::fs::create_dir_all(&upload_dir).await.ok();
    
    let mut uploaded_files = Vec::new();
    
    for file in files {
        match file.save_with_unique_name(&upload_dir).await {
            Ok(path) => {
                uploaded_files.push(serde_json::json!({
                    "filename": file.filename,
                    "size": file.size,
                    "content_type": file.content_type,
                    "path": path.to_string_lossy(),
                }));
            }
            Err(e) => {
                return json!({"error": format!("Failed to save file: {}", e)});
            }
        }
    }
    
    json!({
        "success": true,
        "description": description,
        "count": uploaded_files.len(),
        "files": uploaded_files,
    })
}

#[post("/upload/image")]
async fn upload_image(form: FormData) -> Response {
    let file = match form.file("image") {
        Some(f) => f,
        None => return json!({"error": "No image uploaded"}),
    };
    
    // Validate it's an image
    if !file.is_image() {
        return json!({"error": "File is not an image"});
    }
    
    // Validate config
    let config = UploadConfig {
        max_file_size: 5 * 1024 * 1024, // 5MB
        allowed_extensions: Some(vec!["jpg".to_string(), "jpeg".to_string(), "png".to_string(), "gif".to_string()]),
        allowed_mime_types: Some(vec!["image/".to_string()]),
        upload_dir: PathBuf::from("uploads/images"),
    };
    
    if let Err(e) = config.validate(file) {
        return json!({"error": e.to_string()});
    }
    
    config.ensure_upload_dir().await.ok();
    
    let caption = form.field("caption").unwrap_or("No caption");
    
    match file.save_with_unique_name(&config.upload_dir).await {
        Ok(path) => json!({
            "success": true,
            "caption": caption,
            "filename": file.filename,
            "size": file.size,
            "content_type": file.content_type,
            "path": path.to_string_lossy(),
        }),
        Err(e) => json!({"error": format!("Failed to save image: {}", e)}),
    }
}

#[get("/uploads/:filename")]
async fn get_uploaded_file(Path(filename): Path<String>) -> Response {
    let path = PathBuf::from("uploads").join(&filename);
    
    match tokio::fs::read(&path).await {
        Ok(data) => {
            let mut res = Response::new(StatusCode::Ok, data);
            
            // Try to set content type based on extension
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let content_type = match ext {
                    "jpg" | "jpeg" => "image/jpeg",
                    "png" => "image/png",
                    "gif" => "image/gif",
                    "pdf" => "application/pdf",
                    "txt" => "text/plain",
                    _ => "application/octet-stream",
                };
                res.headers.insert("Content-Type".to_string(), content_type.to_string());
            }
            
            res
        }
        Err(_) => {
            let mut res = Response::new(StatusCode::NotFound, b"File not found");
            res.headers.insert("Content-Type".to_string(), "text/plain".to_string());
            res
        }
    }
}

#[tokio::main]
async fn main() {
    println!("🔥 Firework File Upload Example");
    println!("");
    
    // Create uploads directory
    tokio::fs::create_dir_all("uploads").await.ok();
    tokio::fs::create_dir_all("uploads/images").await.ok();
    
    println!("✅ Upload directories created");
    println!("");
    println!("Open: http://localhost:8080");
    println!("");
    println!("Test with cURL:");
    println!(r#"  curl -F "title=Test" -F "file=@/path/to/file.pdf" http://localhost:8080/upload"#);
    println!(r#"  curl -F "files=@file1.txt" -F "files=@file2.txt" http://localhost:8080/upload/multiple"#);
    println!(r#"  curl -F "caption=My photo" -F "image=@photo.jpg" http://localhost:8080/upload/image"#);
    println!("");
    println!("Uploaded files will be saved to:");
    println!("  - uploads/ (general files)");
    println!("  - uploads/images/ (images)");
    println!("");
    
    routes!()
        .listen("127.0.0.1:8080")
        .await
        .expect("Failed to start server");
}
