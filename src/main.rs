use clap::Parser;

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

    // TODO: Load wordlist, generate URLs, probe them asynchronously
    // TODO: If AI is enabled, analyze responses with heuristics/ML
    // For now, this is a placeholder—let's build it out step by step!
}
