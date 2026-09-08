use mcsm_protocol::error::{McsmError, McsmResult};
use reqwest::multipart;
use std::path::Path;
use tokio::fs;

pub async fn upload_file(passport_url: &str, local_path: &Path) -> McsmResult<()> {
    let bytes = fs::read(local_path)
        .await
        .map_err(|e| McsmError::Io(e.to_string()))?;
    let file_name = local_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("upload.bin")
        .to_string();
    let part = multipart::Part::bytes(bytes).file_name(file_name);
    let form = multipart::Form::new().part("file", part);
    let client = reqwest::Client::new();
    let resp = client
        .post(passport_url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| McsmError::Network(e.to_string()))?;
    if !resp.status().is_success() {
        let status = resp.status().as_u16() as i32;
        let text = resp.text().await.unwrap_or_default();
        return Err(McsmError::Api {
            status,
            message: text,
        });
    }
    Ok(())
}

pub async fn download_file(passport_url: &str, dest: &Path) -> McsmResult<()> {
    let client = reqwest::Client::new();
    let resp = client
        .get(passport_url)
        .send()
        .await
        .map_err(|e| McsmError::Network(e.to_string()))?;
    if !resp.status().is_success() {
        let status = resp.status().as_u16() as i32;
        let text = resp.text().await.unwrap_or_default();
        return Err(McsmError::Api {
            status,
            message: text,
        });
    }
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| McsmError::Network(e.to_string()))?;
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| McsmError::Io(e.to_string()))?;
    }
    fs::write(dest, &bytes)
        .await
        .map_err(|e| McsmError::Io(e.to_string()))?;
    Ok(())
}
