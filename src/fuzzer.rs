use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use tokio::task;

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
    let mut handles = vec![];

    // Chunk the URLs to control concurrency (e.g., 10 at a time)
    for chunk in urls.chunks(concurrency) {
        for url in chunk {
            let url_clone = url.clone();
            let handle = task::spawn(async move {
                let result = probe_url(url_clone).await;
                println!("{}", result);  // Output the result
            });
            handles.push(handle);
        }

        // Wait for this batch to finish before next (simple rate limiting)
        for handle in handles.drain(..) {
            if let Err(e) = handle.await {
                eprintln!("Task error: {}", e);
            }
        }
    }

    println!("Fuzzing complete!");
}
