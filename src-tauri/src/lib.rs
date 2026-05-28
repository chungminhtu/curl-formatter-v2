use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Deserialize)]
struct HttpRequest {
    method: String,
    url: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

#[derive(Serialize)]
struct HttpResponse {
    status: u16,
    status_text: String,
    headers: HashMap<String, String>,
    body: String,
    elapsed_ms: u128,
    size_bytes: usize,
}

#[tauri::command]
async fn http_request(req: HttpRequest) -> Result<HttpResponse, String> {
    let start = Instant::now();

    let client = reqwest::Client::builder()
        .user_agent("curl-runner/1.0")
        .build()
        .map_err(|e| e.to_string())?;

    let method = reqwest::Method::from_bytes(req.method.to_uppercase().as_bytes())
        .map_err(|e| e.to_string())?;

    let mut builder = client.request(method, &req.url);

    for (k, v) in &req.headers {
        builder = builder.header(k, v);
    }

    if let Some(body) = req.body {
        if !body.is_empty() {
            builder = builder.body(body);
        }
    }

    let resp = builder.send().await.map_err(|e| e.to_string())?;
    let status = resp.status();
    let status_text = status
        .canonical_reason()
        .unwrap_or("")
        .to_string();

    let mut headers = HashMap::new();
    for (k, v) in resp.headers().iter() {
        headers.insert(
            k.to_string(),
            v.to_str().unwrap_or("").to_string(),
        );
    }

    let body_bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    let size_bytes = body_bytes.len();
    let body = String::from_utf8_lossy(&body_bytes).to_string();

    Ok(HttpResponse {
        status: status.as_u16(),
        status_text,
        headers,
        body,
        elapsed_ms: start.elapsed().as_millis(),
        size_bytes,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![http_request])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
