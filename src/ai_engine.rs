use crate::prober::ProbeResult;
use rust_bert::pipelines::sentiment::{SentimentClassifier, Sentiment};  // For NLP example
use std::sync::OnceLock;  // For lazy model loading

// Lazy-loaded model (loads once)
fn sentiment_model() -> &'static SentimentClassifier {
    static MODEL: OnceLock<SentimentClassifier> = OnceLock::new();
    MODEL.get_or_init(|| SentimentClassifier::new().unwrap())
}

// Analyzes a ProbeResult and returns a score (0.0-1.0) + insights
pub fn analyze(result: &ProbeResult) -> (f32, String) {
    let mut score = 0.0;
    let mut insights = Vec::new();

    // Heuristic rules (simple "AI")
    if result.status == 200 {
        score += 0.5;
        insights.push("Successful response".to_string());
    }
    if let Some(server) = result.headers.get("server") {
        if server.contains("Apache") || server.contains("Nginx") {
            score += 0.2;
            insights.push(format!("Interesting server: {}", server));
        }
    }
    if let Some(title) = &result.title {
        if title.to_lowercase().contains("admin") || title.to_lowercase().contains("login") {
            score += 0.3;
            insights.push("Potential sensitive page".to_string());
        }

        // Real AI: Sentiment analysis on title (e.g., detect negative/error vibes)
        let sentiments: Vec<Sentiment> = sentiment_model().predict(&[title.as_str()]);
        if let Some(sentiment) = sentiments.first() {
            if sentiment.polarity == "negative" && sentiment.score > 0.7 {
                score -= 0.2;  // Downgrade likely error pages
                insights.push(format!("Negative sentiment detected (score: {:.2})", sentiment.score));
            }
        }
    }

    score = score.clamp(0.0, 1.0);  // Normalize
    (score, insights.join("; "))
}
