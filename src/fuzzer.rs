use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task;
use reqwest::Client;
use crate::prober::{probe_url, ProbeResult};
use crate::ai_engine::analyze;
use crate::tech_fingerprinter::fingerprint;
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
    output: String,  // Assuming this is not Option anymore—adjust based on your main.rs
    method: String,
    data: Option<String>,
    raw_headers: Vec<String>,
) {
    let words = match load_wordlist(&wordlist_path) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Error loading wordlist: {}", e);
            return;
        }
    };

    let urls = generate_urls(&base_url, &words);

    // Create a shared reqwest Client for efficient requests
    let client = Client::builder()
        .user_agent("Grimnir/0.1")  // Set a user agent to be polite
        .timeout(std::time::Duration::from_secs(5))  // Basic timeout
        .build()
        .expect("Failed to build reqwest client");

    // Rate limiting: Semaphore limits to 'rate' concurrent requests
    let semaphore = Arc::new(Semaphore::new(rate));
    let method_upper = method.to_uppercase();
    if !["GET", "POST", "HEAD", "PUT"].contains(&method_upper.as_str()) {
        eprintln!("Unsupported method: {}. Falling back to GET.", method);
        let method_upper = "GET".to_string(); }
    let mut targets = vec![];
    for word in &words {
        let fuzzed_url = base_url.replace("FUZZ", word);
        let fuzzed_data = data.as_ref().map(|d| d.replace("FUZZ", word));
        targets.push((fuzzed_url, fuzzed_data)); }
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

    let mut handles = vec![];
    let mut results: Vec<(ProbeResult, Option<(f32, String)>)> = vec![];  // Collect results with optional AI

    // Chunk the URLs to control concurrency (batches of 'concurrency' size)
    for chunk in targets.chunks(concurrency) {
        for (url, opt_data) in chunk {
            let url_clone = url.clone();
            let opt_data_clone = opt_data.clone();
            let client_clone = client.clone();
            let sem_clone = semaphore.clone();
            let method_clone = method_upper.clone();
            let handle = task::spawn(async move {
                let _permit = sem_clone.acquire().await.unwrap();
                match probe_url(url_clone, &client_clone, &method_clone, opt_data_clone).await {
                    Ok(result) => {
                        if should_filter(&result, &filter_status, &filter_size) {
                            return None;
                        }
                        if ai_enabled {
                            let (score, insights) = analyze(&result);
                            Some((result, Some((score, insights))))
                        } else {
                            Some((result, None))
                        }
                    }
                    Err(e) => {
                        eprintln!("Probe error for {}: {}", url_clone, e);
                        None
                    }
                }
            });
            handles.push(handle);
        }
        // Now in the task loop, pass parsed_headers.clone() to probe_url
        // For each task:
        let parsed_headers_clone = parsed_headers.clone();
        let handle = task::spawn(async move {
            // ... acquire permit ...
            match probe_url(url_clone, &client_clone, &method_clone, opt_data_clone, parsed_headers_clone).await {

        // Wait for this batch to finish and collect results
        for handle in handles.drain(..) {
            if let Ok(Some(res)) = handle.await {  // Handle potential task results
                results.push(res);
            } else if let Err(e) = handle.await {
                eprintln!("Task error: {}", e);
            }
        }
    }

    // Output results based on format
    if let Some(out_format) = output {
        if out_format == "json" || out_format == "pretty-json" {
            output_json(&results, out_format == "pretty-json");
        } else {
            eprintln!("Unknown output format: {}. Falling back to terminal.", out_format);
            output_terminal(&results);
        }
    } else {
        output_terminal(&results);
    }

    println!("Fuzzing complete!");
        let tech = if tech_enabled { Some(fingerprint(&result)) } else { None };
}

// Helper to check if result should be filtered
fn should_filter(result: &ProbeResult, filter_status: &Option<Vec<u16>>, filter_size: &Option<Vec<usize>>) -> bool {
    if let Some(statuses) = filter_status {
        if statuses.contains(&result.status) {
            return true;
        }
    }
    if let Some(sizes) = filter_size {
        let body_size = result.title.as_ref().map_or(0, |t| t.len());  // Placeholder; expand to real size later (e.g., add body_len to ProbeResult)
        if sizes.iter().any(|&s| body_size < s) {  // Example: filter if smaller than any value
            return true;
        }
    }
    false
}

// Terminal output (pretty print)
fn output_terminal(results: &[(ProbeResult, Option<(f32, String)>)]) {
    for (result, ai) in results {
        println!("URL: {}", result.url);
        println!("Status: {}", result.status);
        if let Some(t) = tech {
            println!("Detected Tech: {}", t.join(", "));
        }
        if let Some(title) = &result.title {
            println!("Title: {}", title);
        }
        if let Some(server) = result.headers.get("server") {
            println!("Server: {}", server);
        }
        if let Some((score, insights)) = ai {
            println!("AI Score: {:.2}", score);
            println!("Insights: {}", insights);
        }
        println!("---");
    }
}

// JSON output (with AI fields included)
#[derive(Serialize)]
struct JsonResult {
    url: String,
    status: u16,
    headers: std::collections::HashMap<String, String>,
    title: Option<String>,
    ai_score: Option<f32>,
    ai_insights: Option<String>,
}

fn output_json(results: &[(ProbeResult, Option<(f32, String)>)], pretty: bool) {
    let json_results: Vec<JsonResult> = results.iter().map(|(res, ai)| {
        JsonResult {
            url: res.url.clone(),
            status: res.status,
            headers: res.headers.clone(),
            title: res.title.clone(),
            ai_score: ai.map(|(score, _)| score),
            ai_insights: ai.map(|(_, insights)| insights.clone()),
        }
    }).collect();

    let json = if pretty {
        serde_json::to_string_pretty(&json_results).unwrap()
    } else {
        serde_json::to_string(&json_results).unwrap()
    };
    println!("{}", json);
}
