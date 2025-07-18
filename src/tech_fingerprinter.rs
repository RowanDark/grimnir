use crate::prober::ProbeResult;
use lazy_static::lazy_static;
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
        } else if powered_by.contains("Django") {
            detected.insert("Django".to_string());
        }
    }
    if result.headers.contains_key("cf-ray") || result.headers.contains_key("cf-cache-status") {
        detected.insert("Cloudflare".to_string());
    }
    if result.headers.contains_key("x-amz-id-2") || result.headers.contains_key("x-amz-request-id") {
        detected.insert("AWS".to_string());
    }
    if result.headers.contains_key("x-shopify-stage") {
        detected.insert("Shopify".to_string());
    }

    // Cookie-based detection (e.g., for Laravel, Django)
    if let Some(set_cookie) = result.headers.get("set-cookie") {
        if set_cookie.contains("laravel_session") {
            detected.insert("Laravel".to_string());
        }
        if set_cookie.contains("csrftoken") {
            detected.insert("Django".to_string());
        }
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

    // Body snippet-based detection
    if let Some(body) = &result.body_snippet {
        lazy_static! {
            static ref REACT_RE: Regex = Regex::new(r"(?i)react-dom").unwrap();
            static ref JOOMLA_RE: Regex = Regex::new(r"(?i)joomla|com_content").unwrap();
            static ref WP_BODY_RE: Regex = Regex::new(r"(?i)wp-admin|wp-login").unwrap();
        }
        if REACT_RE.is_match(body) {
            detected.insert("React".to_string());
        }
        if JOOMLA_RE.is_match(body) {
            detected.insert("Joomla".to_string());
        }
        if WP_BODY_RE.is_match(body) {
            detected.insert("WordPress".to_string());
        }
    }

    // Return as sorted Vec for consistent output
    let mut sorted: Vec<String> = detected.into_iter().collect();
    sorted.sort();
    sorted
}
