use clap::Parser;
mod fuzzer;  // Import our new module

#[derive(Parser, Debug)]
#[command(name = "grimnir", version = "0.1.0", about = "A fused ffuf + httpx tool with AI smarts")]
struct Args {
    /// Base URL with FUZZ placeholder (e.g., http://example.com/FUZZ)
    #[arg(short = 'u', long)]
    url: String,
    #[arg(short = 'w', long)]
    wordlist: String,
    #[arg(long)]
    ai: bool,
    #[arg(long, value_delimiter = ',', num_args = 1..)]
    filter_status: Option<Vec<u16>>,
    #[arg(long, value_delimiter = ',', num_args = 1..)]
    filter_size: Option<Vec<usize>>,
    #[arg(long, default_value_t = 10)]
    rate: usize,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("Grimnir is ready to probe and fuzz!");
    println!("Target URL: {}", args.url);
    println!("Wordlist: {}", args.wordlist);
    println!("AI enabled: {}", args.ai);

    // Call the fuzzer
    let concurrency = 10;  // Adjustable later via CLI arg
    if args.ai {
        println!("AI mode not yet implemented—running basic fuzz for now!");
        // TODO: Integrate AI analysis here
    }
    fuzzer::fuzz(
        args.url,
        args.wordlist,
        10,  // Concurrency (hardcoded for now)
        args.ai,
        args.filter_status,
        args.filter_size,
        args.rate,
    ).await;
}
