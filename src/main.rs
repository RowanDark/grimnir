use clap::Parser;

// Import modules (explicit for clarity, even if using crate::)
mod ai_engine;
mod fuzzer;
mod prober;
mod tech_fingerprinter;

#[derive(Parser, Debug)]
#[command(name = "grimnir", version = "0.1.0", about = "A fused ffuf + httpx tool with AI smarts")]
struct Args {
    /// Base URL with FUZZ placeholder (e.g., http://example.com/FUZZ)
    #[arg(short = 'u', long)]
    url: String,

    /// Path to wordlist file
    #[arg(short = 'w', long)]
    wordlist: String,

    /// Enable AI enhancements (e.g., response analysis)
    #[arg(long)]
    ai: bool,

    /// Filter out specific status codes (comma-separated)
    #[arg(long, value_delimiter = ',', num_args = 1..)]
    filter_status: Option<Vec<u16>>,

    /// Filter out responses smaller than these sizes (comma-separated)
    #[arg(long, value_delimiter = ',', num_args = 1..)]
    filter_size: Option<Vec<usize>>,

    /// Requests per second (rate limiting)
    #[arg(long, default_value_t = 10)]
    rate: usize,

    /// Output format and optional file (e.g., json:results.json)
    #[arg(long, default_value = "terminal")]
    output: String,

    /// HTTP method (GET, POST, PUT, HEAD)
    #[arg(long, default_value = "GET")]
    method: String,

    /// Optional data/body for POST/PUT (can contain FUZZ)
    #[arg(long)]
    data: Option<String>,

    /// Custom headers (repeatable, format "Key: Value")
    #[arg(short = 'H', long, num_args = 1..)]
    header: Vec<String>,

    /// Enable tech fingerprinting
    #[arg(long)]
    tech: bool,

    /// Proxy URL (e.g., http://proxy:8080)
    #[arg(long)]
    proxy: Option<String>,

    /// Proxy authentication (format "user:pass")
    #[arg(long)]
    proxy_auth: Option<String>,

    /// Regex patterns to filter out responses (repeatable)
    #[arg(long, num_args = 1..)]
    filter_regex: Vec<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!(r#"

  _______ .______       __  .___  ___. .__   __.  __  .______      
 /  _____||   _  \     |  | |   \/   | |  \ |  | |  | |   _  \     
|  |  __  |  |_)  |    |  | |  \  /  | |   \|  | |  | |  |_)  |    
|  | |_ | |      /     |  | |  |\/|  | |  . `  | |  | |      /     
|  |__| | |  |\  \----.|  | |  |  |  | |  |\   | |  | |  |\  \----.
 \______| | _| `._____||__| |__|  |__| |__| \__| |__| | _| `._____|
                                                                   
    By: RowanDark v0.1.0 2025
    "#);
    println!("Grimnir is ready to probe and fuzz!");
    println!("Target URL: {}", args.url);
    println!("Wordlist: {}", args.wordlist);
    println!("AI enabled: {}", args.ai);
    println!("Method: {}", args.method);
    println!("Tech fingerprinting: {}", args.tech);
    if let Some(proxy) = &args.proxy {
        println!("Using proxy: {}", proxy);
    }

    // Call the fuzzer with all params
    let concurrency = 10;  // Hardcoded for now; could make CLI arg later
    fuzzer::fuzz(
        args.url,
        args.wordlist,
        concurrency,
        args.ai,
        args.filter_status,
        args.filter_size,
        args.rate,
        args.output,
        args.method,
        args.data,
        args.header,
        args.tech,
        args.proxy,
        args.proxy_auth,
        args.filter_regex,
    ).await;
}
