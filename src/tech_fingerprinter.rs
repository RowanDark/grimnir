use crate::prober::ProbeResult;
use lazy_static::lazy_static;  // For static regexes (add to Cargo.toml if missing)
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
    if result.headers.contains_key("x-amz-id-2") || result.headers.contains_key("x-amz-request-id") {
        detected.insert("AWS".to_string());
    }

    // Title-based detection
    if let Some(title) = &result.title {
        lazy_static! {
            static ref WP_RE: Regex = Regex::new(r"(?i)wordpress|wp-content").unwrap();
            static ref DRUPAL_RE: Regex = Regex::new(r"(?i)drupal").unwrap();
        }
        if WP_RE.is_match(title) {
            detected.insert("WordPress".to_string());
        }
        if DRUPAL_RE.is_match(title) {
            detected.insert("Drupal".to_string());
        }
    }

    // Body snippet-based detection (enhancement for more context)
    if let Some(body) = &result.body_snippet {
        lazy_static! {
            static ref REACT_RE: Regex = Regex::new(r"(?i)react-dom").unwrap();  // Common React script pattern
            static ref WP_BODY_RE: Regex = Regex::new(r"(?i)wp-admin|wp-login").unwrap();
        }
        if REACT_RE.is_match(body) {
            detected.insert("React".to_string());
        }
        if WP_BODY_RE.is_match(body) {
            detected.insert("WordPress".to_string());
        }
    }

    // TODO: Add more (e.g., cookies for Laravel, body regex for Joomla, or external signature loading)
    detected.into_iter().collect()  // Return as sorted Vec for consistency
}
