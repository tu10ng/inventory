use axum::extract::{Multipart, State};
use axum::Json;
use base64::Engine as _;
use sqlx::SqlitePool;
use std::io::Write;
use std::process::Command;
use tokio::task;

use image::GenericImageView;

use crate::error::AppError;
use crate::handlers::ai;

/// 压缩/缩放图片以减少视觉 API 的传输和处理延迟。
/// 长边最大 2048px，超出则等比缩放；统一输出 JPEG 质量 85。
/// 解码/编码失败时原样返回原始数据，不影响功能。
fn preprocess_image(data: &[u8]) -> Vec<u8> {
    const MAX_DIM: u32 = 2048;

    let img = match image::load_from_memory(data) {
        Ok(img) => img,
        Err(_) => return data.to_vec(),
    };

    let (w, h) = img.dimensions();
    if w <= MAX_DIM && h <= MAX_DIM {
        let mut buf = std::io::Cursor::new(Vec::new());
        if img.write_to(&mut buf, image::ImageFormat::Jpeg).is_ok() {
            return buf.into_inner();
        }
        return data.to_vec();
    }

    let ratio = MAX_DIM as f64 / w.max(h) as f64;
    let new_w = (w as f64 * ratio) as u32;
    let new_h = (h as f64 * ratio) as u32;
    let resized = img.resize_exact(new_w, new_h, image::imageops::FilterType::Lanczos3);

    let mut buf = std::io::Cursor::new(Vec::new());
    match resized.write_to(&mut buf, image::ImageFormat::Jpeg) {
        Ok(_) => buf.into_inner(),
        Err(_) => data.to_vec(),
    }
}

/// Detect MIME type from file extension and magic bytes.
fn detect_mime(name: &str, data: &[u8]) -> &'static str {
    // Check magic bytes first
    if data.len() >= 3 && &data[0..3] == b"\xff\xd8\xff" {
        return "image/jpeg";
    }
    if data.len() >= 4 && &data[0..4] == b"\x89PNG" {
        return "image/png";
    }
    if data.len() >= 4 && &data[0..4] == b"RIFF" && data.len() >= 12 && &data[8..12] == b"WEBP" {
        return "image/webp";
    }
    if data.len() >= 4 && &data[0..4] == b"GIF8" {
        return "image/gif";
    }

    // Fallback to extension
    let ext = std::path::Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext.to_lowercase().as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        _ => "image/png", // safest default for base64 data URL
    }
}

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

/// AI Vision recognition: send images to a multimodal LLM (task="ocr")
/// and return the recognized text. Falls back gracefully if no vision model
/// is configured.
pub async fn ocr_vision(
    State(pool): State<SqlitePool>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    // Collect all image data into memory (multipart is !Send)
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

    // Load vision model config (task="ocr")
    let config = ai::load_llm_config(&pool, "ocr").await
        .map_err(|_| AppError::bad_request(
            "未配置 AI 视觉识别模型。请在设置页 → LLM 配置中，为 task=\"ocr\" 配置一个多模态模型（如 moonshot-v1-8k-vision）并设为活跃。",
        ))?;

    // Preprocess images (resize/compress) before base64 encoding
    let images: Vec<(String, Vec<u8>)> = images
        .into_iter()
        .map(|(name, data)| (name, preprocess_image(&data)))
        .collect();

    // Base64-encode all images
    let engine = base64::engine::general_purpose::STANDARD;
    let encoded: Vec<(String, String)> = images
        .iter()
        .map(|(name, data)| {
            let mime = detect_mime(name, data);
            let b64 = engine.encode(data);
            (mime.to_string(), b64)
        })
        .collect();

    // Build vision prompt
    let system_prompt = "你是一个订单图片识别助手。用户会上传订单截图或商品图片，请仔细识别图片中的所有商品信息。";

    let user_prompt = concat!(
        "请识别图片中的商品信息，按以下格式提取每个商品：\n\n",
        "- 商品名称（name）：去除冗余描述（如「官方正品」「限时特价」等促销语）\n",
        "- 品牌（brand）：品牌名称。如果是迪卡侬子品牌（QUECHUA/FORCLAZ/VAN RYSEL/TRIBAN/ROCKRIDER/DOMYOS/KIPSTA/SOLOGNAC/CAPERLAN/OLAIAN/INESIS/NABAIJI/SUBEA/OXYLANE）请在品牌前列「迪卡侬」作为母品牌\n",
        "- 型号（model）：商品具体型号/货号\n",
        "- 数量（default_qty）：购买数量\n",
        "- 价格：识别价格时注意单位，如果价格以「分」为单位（如 2990 分），转换为「元」（29.90 元）\n",
        "- 颜色/尺码等规格信息\n\n",
        "请直接输出识别的商品列表，每个商品用一行描述，格式：名称 | 品牌 | 型号 | 数量 | 价格 | 规格\n",
        "如果没有识别到某个字段，写「未知」即可。",
    );

    let text = ai::call_llm_vision(&config, system_prompt, user_prompt, &encoded).await
        .map_err(|e| AppError::bad_request(format!("AI 视觉识别失败: {:#?}", e)))?;

    Ok(Json(serde_json::json!({ "ocr_text": text })))
}
