use clap::Parser;
mod fuzzer;  // Import our new module

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
    fuzzer::fuzz(args.url, args.wordlist, concurrency).await;
}
