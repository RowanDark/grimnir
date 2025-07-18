use crate::prober::ProbeResult;
use rust_bert::pipelines::sentiment::{SentimentClassifier, Sentiment};
use std::sync::OnceLock;

// Lazy-loaded model (loads once, with fallback)
fn sentiment_model() -> Option<&'static SentimentClassifier> {
    static MODEL: OnceLock<Option<SentimentClassifier>> = OnceLock::new();
    MODEL.get_or_init(|| {
        match SentimentClassifier::new() {
            Ok(model) => Some(model),
            Err(e) => {
                eprintln!("Failed to load sentiment model: {}. AI sentiment disabled.", e);
                None
            }
        }
    }).as_ref()  // Return Option<&'static>
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
        if server.contains("Apache") || server.contains("Nginx") || server.contains("IIS") {
            score += 0.2;
            insights.push(format!("Interesting server: {}", server));
        }
    }
    if let Some(title) = &result.title {
        if title.to_lowercase().contains("admin") || title.to_lowercase().contains("login") {
            score += 0.3;
            insights.push("Potential sensitive page".to_string());
        }

        // Real AI: Sentiment analysis on title (skip if empty)
        if !title.is_empty() {
            if let Some(model) = sentiment_model() {
                let sentiments: Vec<Sentiment> = model.predict(&[title.as_str()]);
                if let Some(sentiment) = sentiments.first() {
                    if sentiment.polarity == "negative" && sentiment.score > 0.7 {
                        score -= 0.2;  // Downgrade likely error pages
                        insights.push(format!("Negative sentiment in title (score: {:.2})", sentiment.score));
                    }
                }
            }
        }
    }

    // Enhancement: If body_snippet exists, run sentiment on it for more context
    if let Some(body) = &result.body_snippet {
        if !body.is_empty() {
            if let Some(model) = sentiment_model() {
                let sentiments: Vec<Sentiment> = model.predict(&[body.as_str()]);
                if let Some(sentiment) = sentiments.first() {
                    if sentiment.polarity == "negative" && sentiment.score > 0.8 {  // Slightly higher threshold for body
                        score -= 0.1;  // Mild downgrade
                        insights.push(format!("Negative sentiment in body snippet (score: {:.2})", sentiment.score));
                    }
                }
            }
        }
    }

    score = score.clamp(0.0, 1.0);  // Normalize
    (score, insights.join("; "))
}
