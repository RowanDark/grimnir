use reqwest::{Client, Error as ReqwestError};
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;

// Custom struct for probe results—easy to expand and serialize
#[derive(Debug, Serialize)]
pub struct ProbeResult {
    pub url: String,
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub title: Option<String>,
    pub body_snippet: Option<String>,  // First 1KB of body for fingerprinting/AI
}

// Async function to probe a URL and parse response
pub async fn probe_url(
    url: String,
    client: &Client,
    method: &str,
    data: Option<String>,
    custom_headers: Vec<(String, String)>,  // Custom headers to attach
) -> Result<ProbeResult, ReqwestError> {
    let mut request = match method {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "HEAD" => client.head(&url),
        _ => client.get(&url),  // Fallback
    };

    // Attach custom headers first (allows overriding defaults)
    for (key, value) in custom_headers {
        request = request.header(key, value);
    }

    // Attach data for methods that support bodies (POST, PUT)
    if matches!(method, "POST" | "PUT") {
        if let Some(body) = data {
            let content_type = if body.starts_with('{') || body.starts_with('[') {
                "application/json"
            } else if body.contains('=') {
                "application/x-www-form-urlencoded"
            } else {
                "text/plain"
            };
            request = request.header("Content-Type", content_type).body(body);
        }
    }

    let res = request.send().await?;

    let status = res.status().as_u16();

    // Collect headers into a HashMap (lowercase keys for consistency)
    let mut headers = HashMap::new();
    for (key, value) in res.headers() {
        if let Ok(val_str) = value.to_str() {
            headers.insert(key.as_str().to_lowercase(), val_str.to_string());
        }
    }

    // Extract body only if not HEAD (HEAD has no body)
    let body = if method != "HEAD" { res.text().await? } else { String::new() };
    let body_snippet = if !body.is_empty() { Some(body.chars().take(1024).collect()) } else { None };

    // Parse title with regex (assumes HTML)
    let title_re = Regex::new(r"<title>(.*?)</title>").unwrap();
    let title = title_re.captures(&body).and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()));

    Ok(ProbeResult {
        url,
        status,
        headers,
        title,
        body_snippet,
    })
}
