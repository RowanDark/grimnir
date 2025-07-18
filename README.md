 Grimnir

Grimnir is a Rust-based cybersecurity tool that combines the fuzzing capabilities of [ffuf](https://github.com/ffuf/ffuf) with the HTTP probing of [httpx](https://github.com/projectdiscovery/httpx), enhanced with AI for smarter analysis. It's designed for ethical reconnaissance—use only on targets you own or have explicit permission for.

## Features (Planned)
- **Fuzzing**: Blast URLs with wordlists to discover hidden paths.
- **Probing**: Analyze HTTP responses for status, headers, tech stack, etc.
- **AI Integration**: Prioritize findings, detect anomalies, or extract insights from responses.
- Fast, concurrent, and safe thanks to Rust.

This is a work-in-progress—contributions welcome!

## Setup Instructions

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (install via rustup).
- Git for cloning the repo.

### Installation
1. Clone the repo:
git clone https://github.com/rowandark/grimnir.git
cd grimnir

2. Build the project:
cargo build --release

3. Run it (example):
./target/release/grimnir -u "http://example.com/FUZZ" -w /path/to/wordlist.txt --ai

### Dependencies
- Managed via Cargo—run `cargo build` to fetch them.
- For AI features, you may need additional setup (e.g., models for rust-bert).

## Usage
- Basic fuzz: `grimnir -u "http://target/FUZZ" -w wordlist.txt`
- With AI: Add `--ai` for enhanced analysis.
- More flags coming soon!

## Development
- Edit `src/main.rs` for CLI logic.
- Add modules in `src/` for fuzzer, prober, ai_engine, etc.
- Test with `cargo test` (add tests as we go).

## Ethics and Legal
Always obtain permission before scanning. This tool is for educational and defensive purposes only.

## License
MIT
