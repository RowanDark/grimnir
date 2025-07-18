use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use tokio::task;
use reqwest::Client;
use crate::prober::{probe_url, ProbeResult};
use crate::ai_engine::analyze;

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

// Placeholder probe function—simulates an HTTP request and returns a simple result
// (We'll replace this with real reqwest calls in prober.rs later)
async fn probe_url(url: String) -> String {
    // Simulate a request (in reality, use reqwest here)
    // For now, just echo the URL with a fake status
    format!("Probed {} - Status: 200 (placeholder)", url)
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
) {
    let words = match load_wordlist(&wordlist_path) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Error loading wordlist: {}", e);
            return;
        }
    };

    let urls = generate_urls(&base_url, &words);

     let client = Client::builder()
        .user_agent("Grimnir/0.1")  // Set a user agent to be polite
        .timeout(std::time::Duration::from_secs(5))  // Basic timeout
        .build()
        .expect("Failed to build reqwest client");
    let semaphore = Arc::new(Semaphore::new(rate));
    let mut handles = vec![];
    

    // Chunk the URLs to control concurrency 
 for chunk in urls.chunks(concurrency) {
        for url in chunk {
            let url_clone = url.clone();
            let client_clone = client.clone();  
            let handle = task::spawn(async move {
                let _permit = sem_clone.acquire().await.unwrap();
                match probe_url(url_clone, &client_clone).await {
                    Ok(result) => {
                        if should_filter(&result, &filter_status, &filter_size) {
                            return; {
                        if ai_enabled {
                            let (score, insights) = analyze(&result);
                            print_result(&result, Some((score, insights)));
                        } else {
                            print_result(&result, None);
            }
        }
        Err(e) => eprintln!("Probe error for {}: {}", url_clone, e),
    }
            });
            handles.push(handle);
        }

        // Wait for this batch to finish
        for handle in handles.drain(..) {
            if let Err(e) = handle.await {
                eprintln!("Task error: {}", e);
            }
        }
    }

    println!("Fuzzing complete!");
}

fn print_result(result: &ProbeResult, ai: Option<(f32, String)>) {
    println!("URL: {}", result.url);
    println!("Status: {}", result.status);
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
fn should_filter(result: &ProbeResult, filter_status: &Option<Vec<u16>>, filter_size: &Option<Vec<usize>>) -> bool {
    if let Some(statuses) = filter_status {
        if statuses.contains(&result.status) {
            return true;
        }
    }
    if let Some(sizes) = filter_size {
        let body_size = result.title.as_ref().map_or(0, |t| t.len());  // Placeholder; expand to real size later
        if sizes.iter().any(|&s| body_size < s) {  // Example: filter if smaller than any value
            return true;
        }
    }
    false
}
