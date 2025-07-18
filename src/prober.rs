use reqwest::{Client, Error as ReqwestError};
use regex::Regex;
use std::collections::HashMap;

// Custom struct for probe results—easy to expand and serialize
#[derive(Debug)]
pub struct ProbeResult {
    pub url: String,
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub title: Option<String>,  // Extracted from <title> tag
    // TODO: Add more like body length, tech detection
}

// Async function to probe a URL and parse response
pub async fn probe_url(url: String, client: &Client) -> Result<ProbeResult, ReqwestError> {
    let res = client.get(&url).send().await?;

    let status = res.status().as_u16();

    // Collect headers into a HashMap (lowercase keys for consistency)
    let mut headers = HashMap::new();
    for (key, value) in res.headers() {
        if let Ok(val_str) = value.to_str() {
            headers.insert(key.as_str().to_lowercase(), val_str.to_string());
        }
    }

    // Extract body and parse title (simple regex—assumes HTML)
    let body = res.text().await?;
    let title_re = Regex::new(r"<title>(.*?)</title>").unwrap();
    let title = title_re.captures(&body).and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()));

    Ok(ProbeResult {
        url,
        status,
        headers,
        title,
    })
}
