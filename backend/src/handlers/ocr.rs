use axum::extract::Multipart;
use axum::Json;
use std::io::Write;
use std::process::Command;
use tokio::task;

use crate::error::AppError;

/// Run Tesseract OCR on a single image file, returns text or error.
fn tesseract_ocr(image_path: &str) -> Result<String, String> {
    let output = Command::new("tesseract")
        .arg(image_path)
        .arg("stdout")
        .arg("-l")
        .arg("chi_sim+eng")
        .output()
        .map_err(|e| format!("Failed to run tesseract: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Tesseract error: {}", stderr));
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(text)
}

pub async fn ocr_images(
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    // Check tesseract availability
    let tesseract_ok = Command::new("which")
        .arg("tesseract")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !tesseract_ok {
        return Err(AppError::bad_request(
            "Tesseract OCR 未安装。请在服务器上安装 tesseract: pacman -S tesseract tesseract-data-chi_sim tesseract-data-eng",
        ));
    }

    // Collect all image data into memory first (multipart is !Send)
    let mut images: Vec<(String, Vec<u8>)> = Vec::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field
            .file_name()
            .unwrap_or("image.png")
            .to_string();
        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::bad_request(format!("Failed to read uploaded file: {}", e)))?;

        if data.is_empty() {
            continue;
        }
        images.push((name, data.to_vec()));
    }

    if images.is_empty() {
        return Err(AppError::bad_request("未上传任何图片"));
    }

    // Write images to temp files and run tesseract in parallel
    let total = images.len();
    let mut handles = Vec::new();

    for (i, (name, data)) in images.into_iter().enumerate() {
        let handle = task::spawn_blocking(move || -> Result<String, String> {
            let ext = std::path::Path::new(&name)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("png");
            let tmp_path = std::env::temp_dir().join(format!("ocr_import_{}_{}.{}", std::process::id(), i, ext));
            let tmp_str = tmp_path.to_string_lossy().to_string();

            {
                let mut f = std::fs::File::create(&tmp_path)
                    .map_err(|e| format!("Failed to create temp file: {}", e))?;
                f.write_all(&data)
                    .map_err(|e| format!("Failed to write temp file: {}", e))?;
            }

            let text = tesseract_ocr(&tmp_str);

            // Clean up temp file
            let _ = std::fs::remove_file(&tmp_path);

            text
        });
        handles.push(handle);
    }

    // Collect results
    let mut results = Vec::with_capacity(total);
    for handle in handles {
        match handle.await.unwrap_or_else(|e| Err(format!("Task panicked: {}", e))) {
            Ok(text) => results.push(text),
            Err(e) => results.push(format!("[OCR 错误: {}]", e)),
        }
    }

    let ocr_text = results.join("\n---\n");

    Ok(Json(serde_json::json!({ "ocr_text": ocr_text })))
}
