use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task;
use reqwest::{Client, Proxy as ReqwestProxy};
use crate::ai_engine::analyze;
use crate::prober::{probe_url, ProbeResult};
use crate::tech_fingerprinter::fingerprint;
use chrono::Local;  // For timestamped filenames (add to Cargo.toml if missing)
use regex::Regex;
use serde::Serialize;
use serde_json;

// Loads words from a file into a Vec<String>
pub fn load_wordlist(path: &str) -> io::Result<Vec<String>> {
    let path = Path::new(path);
    let file = File::open(path)?;
    let lines = io::BufReader::new(file).lines();
    lines.collect()  // Collects into Vec<Result<String>>, but ? handles errors
}

// Generates fuzzed URLs by replacing "FUZZ" in the base URL
pub fn generate_urls(base_url: &str, words: &[String]) -> Vec<String> {
    words.iter().map(|word| base_url.replace("FUZZ", word)).collect()
}

// Main fuzz function: generates URLs, spawns async tasks for probing
pub async fn fuzz(
    base_url: String,
    wordlist_path: String,
    concurrency: usize,
    ai_enabled: bool,
    filter_status: Option<Vec<u16>>,
    filter_size: Option<Vec<usize>>,
    rate: usize,
    output: String,
    method: String,
    data: Option<String>,
    raw_headers: Vec<String>,
    tech_enabled: bool,
    proxy_url: Option<String>,
    proxy_auth: Option<String>,
    filter_regex: Vec<String>,
) {
    let words = match load_wordlist(&wordlist_path) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Error loading wordlist: {}", e);
            return;
        }
    };

    // Parse raw headers into key-value pairs
    let mut parsed_headers: Vec<(String, String)> = vec![];
    for h in raw_headers {
        let parts: Vec<&str> = h.splitn(2, ':').collect();
        if parts.len() == 2 {
            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();
            parsed_headers.push((key, value));
        } else {
            eprintln!("Invalid header format '{}'. Skipping. Use 'Key: Value'.", h);
        }
    }

    // Compile regex filters
    let mut compiled_regexes: Vec<Regex> = vec![];
    for pattern in filter_regex {
        match Regex::new(&pattern) {
            Ok(re) => compiled_regexes.push(re),
            Err(e) => eprintln!("Invalid regex pattern '{}': {}. Skipping.", pattern, e),
        }
    }

    // Validate and uppercase method
    let mut method_upper = method.to_uppercase();
    if !["GET", "POST", "HEAD", "PUT"].contains(&method_upper.as_str()) {
        eprintln!("Unsupported method: {}. Falling back to GET.", method);
        method_upper = "GET".to_string();
    }

    // Generate targets (fuzzed URL and optional data)
    let mut targets = vec![];
    for word in &words {
        let fuzzed_url = base_url.replace("FUZZ", word);
        let fuzzed_data = data.as_ref().map(|d| d.replace("FUZZ", word));
        targets.push((fuzzed_url, fuzzed_data));
    }

    // Build reqwest client with proxy if provided
    let mut client_builder = Client::builder()
        .user_agent("Grimnir/0.1")
        .timeout(std::time::Duration::from_secs(5));

    if let Some(proxy_str) = proxy_url {
        let proxy_res = if proxy_str.starts_with("socks5://") {
            ReqwestProxy::all(&proxy_str)
        } else if proxy_str.starts_with("https://") {
            ReqwestProxy::https(&proxy_str)
        } else {
            ReqwestProxy::http(&proxy_str)
        };

        match proxy_res {
            Ok(mut proxy) => {
                if let Some(auth) = proxy_auth {
                    let parts: Vec<&str> = auth.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        proxy = proxy.basic_auth(parts[0], parts[1]);
                    } else {
                        eprintln!("Invalid proxy_auth format '{}'. Skipping auth.", auth);
                    }
                }
                client_builder = client_builder.proxy(proxy);
            }
            Err(e) => eprintln!("Failed to set proxy '{}': {}. Continuing without.", proxy_str, e),
        }
    }

    let client = client_builder.build().expect("Failed to build reqwest client");

    // Rate limiting semaphore
    let semaphore = Arc::new(Semaphore::new(rate));

    let mut handles = vec![];
    let mut results: Vec<(ProbeResult, Option<(f32, String)>, Option<Vec<String>>)> = vec![];  // Collect with AI and tech

    // Chunk and process targets
    for chunk in targets.chunks(concurrency) {
        for (url, opt_data) in chunk {
            let url_clone = url.clone();
            let opt_data_clone = opt_data.clone();
            let client_clone = client.clone();
            let sem_clone = semaphore.clone();
            let method_clone = method_upper.clone();
            let parsed_headers_clone = parsed_headers.clone();
            let compiled_regexes_clone = compiled_regexes.clone();  // Clone for task
            let handle = task::spawn(async move {
                let _permit = sem_clone.acquire().await.unwrap();
                match probe_url(url_clone.clone(), &client_clone, &method_clone, opt_data_clone, parsed_headers_clone).await {
                    Ok(result) => {
                        if should_filter(&result, &filter_status, &filter_size, &compiled_regexes_clone) {
                            return None;
                        }
                        let ai_opt = if ai_enabled { Some(analyze(&result)) } else { None };
                        let tech_opt = if tech_enabled { Some(fingerprint(&result)) } else { None };
                        Some((result, ai_opt, tech_opt))
                    }
                    Err(e) => {
                        eprintln!("Probe error for {}: {}", url_clone, e);
                        None
                    }
                }
            });
            handles.push(handle);
        }

        // Wait for batch and collect
        for handle in handles.drain(..) {
            if let Ok(Some(res)) = handle.await {
                results.push(res);
            } else if let Err(e) = handle.await {
                eprintln!("Task error: {}", e);
            }
        }
    }

    // Parse output arg (e.g., "json:output.json")
    let parts: Vec<&str> = output.splitn(2, ':').collect();
    let out_format = parts[0].to_lowercase();
    let out_path = parts.get(1).map(|&s| s.to_string());

    // Generate content based on format
    let content = match out_format.as_str() {
        "json" => Some(serde_json::to_string(&build_json_results(&results)).unwrap()),
        "pretty-json" => Some(serde_json::to_string_pretty(&build_json_results(&results)).unwrap()),
        "terminal" => {
            output_terminal(&results);
            None
        }
        _ => {
            eprintln!("Unknown format '{}'. Using terminal.", out_format);
            output_terminal(&results);
            None
        }
    };

    // Write to file if path provided
    if let Some(mut path) = out_path {
        if path.is_empty() {
            // Auto-generate timestamped filename if no path
            path = format!("grimnir_{}.txt", Local::now().format("%Y%m%d_%H%M%S"));
        }
        let mut file = match File::create(&path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to create file '{}': {}", path, e);
                return;
            }
        };

        if let Some(c) = content {
            if let Err(e) = file.write_all(c.as_bytes()) {
                eprintln!("Failed to write to '{}': {}", path, e);
            } else {
                println!("Output saved to '{}'", path);
            }
        } else {
            // Plain text file output
            let mut text = String::new();
            for (result, ai_opt, tech_opt) in &results {
                text.push_str(&format!("URL: {}\nStatus: {}\n", result.url, result.status));
                if let Some(title) = &result.title {
                    text.push_str(&format!("Title: {}\n", title));
                }
                if let Some(server) = result.headers.get("server") {
                    text.push_str(&format!("Server: {}\n", server));
                }
                if let Some((score, insights)) = ai_opt {
                    text.push_str(&format!("AI Score: {:.2}\nInsights: {}\n", score, insights));
                }
                if let Some(tech) = tech_opt {
                    text.push_str(&format!("Detected Tech: {}\n", tech.join(", ")));
                }
                text.push_str("---\n");
            }
            if let Err(e) = file.write_all(text.as_bytes()) {
                eprintln!("Failed to write to '{}': {}", path, e);
            } else {
                println!("Output saved to '{}'", path);
            }
        }
    } else if let Some(c) = content {
        // Print to stdout if no file but format specified
        println!("{}", c);
    }

    println!("Fuzzing complete!");
}

// Helper to check if result should be filtered
fn should_filter(
    result: &ProbeResult,
    filter_status: &Option<Vec<u16>>,
    filter_size: &Option<Vec<usize>>,
    filter_regexes: &[Regex],
) -> bool {
    // Status filter
    if let Some(statuses) = filter_status {
        if statuses.contains(&result.status) {
            return true;
        }
    }

    // Size filter (using title length as placeholder; update if adding body_len)
    if let Some(sizes) = filter_size {
        let body_size = result.title.as_ref().map_or(0, |t| t.len());
        if sizes.iter().any(|&s| body_size < s) {
            return true;
        }
    }

    // Regex filters (filter out if any matches)
    for re in filter_regexes {
        if re.is_match(&result.url) {
            return true;
        }
        if let Some(title) = &result.title {
            if re.is_match(title) {
                return true;
            }
        }
        if let Some(body) = &result.body_snippet {
            if re.is_match(body) {
                return true;
            }
        }
        if let Some(server) = result.headers.get("server") {
            if re.is_match(server) {
                return true;
            }
        }
    }

    false
}

// Terminal output (pretty print)
fn output_terminal(results: &[(ProbeResult, Option<(f32, String)>, Option<Vec<String>>)]) {
    for (result, ai_opt, tech_opt) in results {
        println!("URL: {}", result.url);
        println!("Status: {}", result.status);
        if let Some(title) = &result.title {
            println!("Title: {}", title);
        }
        if let Some(server) = result.headers.get("server") {
            println!("Server: {}", server);
        }
        if let Some((score, insights)) = ai_opt {
            println!("AI Score: {:.2}", score);
            println!("Insights: {}", insights);
        }
        if let Some(tech) = tech_opt {
            println!("Detected Tech: {}", tech.join(", "));
        }
        println!("---");
    }
}

// JSON output (with AI and tech fields)
#[derive(Serialize)]
struct JsonResult {
    url: String,
    status: u16,
    headers: std::collections::HashMap<String, String>,
    title: Option<String>,
    ai_score: Option<f32>,
    ai_insights: Option<String>,
    detected_tech: Option<Vec<String>>,
}

fn build_json_results(results: &[(ProbeResult, Option<(f32, String)>, Option<Vec<String>>)]) -> Vec<JsonResult> {
    results.iter().map(|(res, ai_opt, tech_opt)| {
        JsonResult {
            url: res.url.clone(),
            status: res.status,
            headers: res.headers.clone(),
            title: res.title.clone(),
            ai_score: ai_opt.map(|(score, _)| score),
            ai_insights: ai_opt.map(|(_, insights)| insights.clone()),
            detected_tech: tech_opt.clone(),
        }
    }).collect()
}
