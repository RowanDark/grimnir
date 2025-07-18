Grimnir
[![License: MIT](https://img.shields.io/badge://opensource.org/licenses.shields.io/badge/Rust-1.nir is a Rust-based cybersecurity tool that blends the fuzzing power of ffuf with the HTTP probing of httpx, enhanced with AI for smarter analysis. It's designed for ethical reconnaissance, directory discovery, and tech fingerprinting—perfect for bug bounty hunters, pen-testers, or security researchers. Use it responsibly on targets you own or have explicit permission for.

Warning: This tool is for educational and defensive purposes only. Always obtain permission before scanning—misuse can lead to legal issues.

Features
Fuzzing: Generate and test URLs from wordlists (replaces "FUZZ" placeholders).

HTTP Probing: Supports GET, POST, PUT, HEAD with response parsing (status, headers, title, body snippet).

AI Enhancements: Heuristic scoring and NLP sentiment analysis on responses (e.g., detect sensitive pages or errors).

Tech Fingerprinting: Detects servers (Apache/Nginx), CMS (WordPress/Drupal), frameworks (PHP/Node.js), and more via headers/body.

Filters: Status codes, response sizes, and custom regex patterns to reduce noise.

Custom Headers: Add auth tokens, cookies, etc., with -H "Key: Value".

Proxy Support: Route through HTTP/HTTPS/SOCKS5 proxies for anonymity.

Output Options: Terminal, JSON (plain/pretty), or files (with auto-timestamping).

Rate Limiting: Control requests per second to avoid detection or overload.

Fast, concurrent, and memory-safe thanks to Rust.

This is a work-in-progress—contributions welcome! See Development below.

Setup Instructions
Prerequisites
Rust (install via rustup; version 1.70+ recommended).

Git for cloning the repo.

For AI features: Ensure libtorch is installed (required for rust-bert/tch; see rust-bert docs for setup).

Installation
Clone the repo:

text
git clone https://github.com/rowandark/grimnir.git
cd grimnir
Build the project:

text
cargo build --release
This fetches dependencies and compiles the binary to target/release/grimnir.

(Optional) Install system deps for AI:

On Ubuntu: sudo apt install libtorch-dev

See tch-rs docs for other OSes if model loading fails.

Run it directly: ./target/release/grimnir --help for CLI options.

Usage
Grimnir runs from the command line. Basic syntax: grimnir -u <URL> -w <WORDLIST> [flags].

Key Flags
-u, --url <URL>: Base URL with "FUZZ" placeholder (required, e.g., "http://example.com/FUZZ").

-w, --wordlist <PATH>: Path to wordlist file (required).

--ai: Enable AI analysis (scoring and insights).

--tech: Enable tech fingerprinting (detects servers/CMS).

--method <METHOD>: HTTP method (GET default; POST, PUT, HEAD supported).

--data <BODY>: Body for POST/PUT (can contain "FUZZ" for fuzzing, e.g., '{"key":"FUZZ"}').

-H, --header <KEY:VALUE>: Custom headers (repeatable, e.g., -H "Authorization: Bearer token").

--filter-status <CODES>: Filter out status codes (comma-separated, e.g., "404,500").

--filter-size <SIZES>: Filter out small responses (comma-separated, e.g., "0,100").

--filter-regex <PATTERN>: Filter out via regex (repeatable, e.g., "--filter-regex 'error'").

--rate <RPS>: Requests per second (default 10).

--output <FORMAT[:FILE]>: Output format (terminal default; e.g., "json:results.json" or "pretty-json").

--proxy <URL>: Proxy server (e.g., "http://proxy:8080" or "socks5://localhost:1080").

--proxy-auth <USER:PASS>: Proxy authentication.

Examples
Basic fuzz and probe:

text
grimnir -u "http://example.com/FUZZ" -w words.txt
With AI and tech detection:

text
grimnir -u "http://target/FUZZ" -w common.txt --ai --tech
POST with fuzzed data and custom headers:

text
grimnir -u "http://api.example.com/FUZZ" -w words.txt --method POST --data '{"param":"FUZZ"}' -H "Content-Type: application/json" -H "Authorization: Bearer token"
Filtered scan with proxy and JSON output to file:

text
grimnir -u "https://secure.site/FUZZ" -w biglist.txt --filter-status 404 --filter-regex "denied" --proxy "socks5://proxy:1080" --output json:scan_results.json
Output example (terminal):

text
URL: http://example.com/admin
Status: 200
Title: Admin Panel
Server: Apache
AI Score: 1.00
Insights: Successful response; Interesting server: Apache; Potential sensitive page
Detected Tech: Apache, PHP
---
For full help: grimnir --help.

Ethics and Legal
Grimnir is built for ethical use—always obtain explicit permission before scanning any target. Features like fuzzing, POST/PUT, and proxies are non-destructive but powerful; misuse (e.g., on unauthorized systems) can violate laws like the CFAA. Avoid sensitive data in commands/logs, and test only on your own environments. The authors are not responsible for misuse.

Development
Source Structure: Edit src/main.rs for CLI, src/fuzzer.rs for core logic, src/prober.rs for HTTP, src/ai_engine.rs for AI, and src/tech_fingerprinter.rs for detections.

Build & Test: cargo build --release and cargo test (add tests as needed).

Contributing: Fork the repo, make changes, and submit a PR. Ideas: More AI models, advanced filters, or GUI integration.

Known Limitations: AI may require model downloads (network-dependent); large wordlists need memory tweaks.

TODOs: Add concurrency as CLI flag, support DELETE/PATCH with safeguards, expand tech signatures.

License
MIT License—see LICENSE for details.

Questions? Open an issue on GitHub or reach out. Happy fuzzing! 🚀
