use axum::extract::Multipart;
use axum::Json;
use std::io::Cursor;
use calamine::Reader;

use crate::error::AppError;
use crate::models::ExcelPreviewResponse;

// ── Excel Preview ──

pub async fn excel_preview(
    mut multipart: Multipart,
) -> Result<Json<ExcelPreviewResponse>, AppError> {
    // Read file from multipart
    let mut file_name = String::new();
    let mut file_data: Vec<u8> = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.file_name().unwrap_or("unknown.xlsx").to_string();
        if !name.is_empty() {
            file_name = name;
        }
        if let Ok(data) = field.bytes().await {
            file_data = data.to_vec();
        }
    }

    if file_data.is_empty() {
        return Err(AppError::bad_request("未上传文件"));
    }

    // Parse with calamine
    let cursor = Cursor::new(file_data);
    let mut workbook: calamine::Xlsx<_> = calamine::open_workbook_from_rs(cursor)
        .map_err(|e| AppError::bad_request(format!("无法解析 Excel 文件: {}", e)))?;

    let sheet_names: Vec<String> = workbook.sheet_names().to_vec();
    if sheet_names.is_empty() {
        return Err(AppError::bad_request("Excel 文件中没有工作表"));
    }

    let active_sheet = sheet_names[0].clone();
    let range = workbook
        .worksheet_range(&active_sheet)
        .map_err(|e| AppError::bad_request(format!("无法读取工作表 '{}': {}", active_sheet, e)))?;

    let mut rows_iter = range.rows();

    // First row as headers, blank cells → "列{N}"
    let first_row: Option<&[calamine::Data]> = rows_iter.next();
    let mut headers: Vec<String> = Vec::new();
    if let Some(row) = first_row {
        for (i, cell) in row.iter().enumerate() {
            let val = cell_to_string(cell).trim().to_string();
            if val.is_empty() {
                headers.push(format!("列{}", i + 1));
            } else {
                headers.push(val);
            }
        }
    }

    // Data rows
    let mut rows: Vec<Vec<String>> = Vec::new();
    for row in rows_iter {
        let row_ref: &[calamine::Data] = row;
        let mut row_data: Vec<String> = Vec::new();
        for cell in row_ref.iter() {
            let val = cell_to_string(cell).trim().to_string();
            row_data.push(val);
        }
        // Pad or truncate to match headers length
        row_data.resize(headers.len(), String::new());
        rows.push(row_data);
    }

    let total_rows = rows.len();

    Ok(Json(ExcelPreviewResponse {
        file_name,
        sheet_names,
        active_sheet,
        headers,
        rows,
        total_rows,
    }))
}

fn cell_to_string(cell: &calamine::Data) -> String {
    match cell {
        calamine::Data::Empty => String::new(),
        calamine::Data::String(s) => s.clone(),
        calamine::Data::Float(f) => {
            if f.fract() == 0.0 {
                format!("{}", *f as i64)
            } else {
                format!("{}", f)
            }
        }
        calamine::Data::Int(i) => format!("{}", i),
        calamine::Data::Bool(b) => format!("{}", b),
        calamine::Data::DateTime(f) => format!("{}", f),
        calamine::Data::DateTimeIso(s) => s.clone(),
        calamine::Data::DurationIso(s) => s.clone(),
        calamine::Data::Error(e) => format!("#ERR:{}", e),
    }
}
