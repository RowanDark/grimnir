use crate::prober::ProbeResult;
use regex::Regex;
use std::collections::HashSet;

// Detects tech based on headers, title, and body patterns
pub fn fingerprint(result: &ProbeResult) -> Vec<String> {
    let mut detected: HashSet<String> = HashSet::new();  // Use set to avoid duplicates

    // Header-based detection
    if let Some(server) = result.headers.get("server") {
        if server.contains("Apache") {
            detected.insert("Apache".to_string());
        } else if server.contains("nginx") {
            detected.insert("Nginx".to_string());
        } else if server.contains("IIS") {
            detected.insert("Microsoft IIS".to_string());
        }
    }
    if let Some(powered_by) = result.headers.get("x-powered-by") {
        if powered_by.contains("PHP") {
            detected.insert("PHP".to_string());
        } else if powered_by.contains("Express") {
            detected.insert("Node.js/Express".to_string());
        }
    }
    if result.headers.contains_key("cf-ray") || result.headers.contains_key("cf-cache-status") {
        detected.insert("Cloudflare".to_string());
    }

    // Title/body-based detection (using title as a proxy for now; expand to full body if needed)
    if let Some(title) = &result.title {
        let wp_re = Regex::new(r"(?i)wordpress|wp-content").unwrap();
        if wp_re.is_match(title) {
            detected.insert("WordPress".to_string());
        }
        let drupal_re = Regex::new(r"(?i)drupal").unwrap();
        if drupal_re.is_match(title) {
            detected.insert("Drupal".to_string());
        }
    }

    // TODO: Add more (e.g., cookies for Laravel, body regex for Joomla)
    detected.into_iter().collect()  // Return as Vec
}
