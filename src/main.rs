use clap::Parser;
mod fuzzer;  // Import our new module

#[derive(Parser, Debug)]
#[command(name = "grimnir", version = "0.1.0", about = "A fused ffuf + httpx tool with AI smarts")]
struct Args {
    // Existing args...
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
    #[arg(long, default_value = "terminal")]
    output: String,
    #[arg(long, default_value = "GET")]
    method: String,
    #[arg(long)]
    data: Option<String>,
    #[arg(short = 'H', long, num_args = 1..)]
    header: Vec<String>,
    #[arg(long)]
    tech: bool,
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
        args.output,
        args.method,
        args.data,
        args.header,
        args.tech,
    ).await;
}
