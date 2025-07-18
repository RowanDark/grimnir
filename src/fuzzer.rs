use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use tokio::task;
use reqwest::Client;
use crate::prober::{probe_url, ProbeResult};

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
pub async fn fuzz(base_url: String, wordlist_path: String, concurrency: usize) {
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
    
    let mut handles = vec![];

    // Chunk the URLs to control concurrency 
 for chunk in urls.chunks(concurrency) {
        for url in chunk {
            let url_clone = url.clone();
            let client_clone = client.clone();  
            let handle = task::spawn(async move {
                match probe_url(url_clone, &client_clone).await {
                    Ok(result) => print_result(&result),  
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

fn print_result(result: &ProbeResult) {
    println!("URL: {}", result.url);
    println!("Status: {}", result.status);
    if let Some(title) = &result.title {
        println!("Title: {}", title);
    }
    if let Some(server) = result.headers.get("server") {
        println!("Server: {}", server);
    }
    println!("---");  // Separator for readability
}
