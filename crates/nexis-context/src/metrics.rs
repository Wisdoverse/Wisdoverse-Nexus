//! Context management metrics
//!
//! Prometheus metrics for monitoring context window management and summarization.

use lazy_static::lazy_static;
use prometheus::{
    register_counter_vec, register_gauge, register_histogram, CounterVec, Gauge, Histogram,
};

lazy_static! {
    // ============================================================================
    // Context Window Metrics
    // ============================================================================

    /// Number of active contexts
    pub static ref CONTEXTS_ACTIVE: Gauge =
        register_gauge!("nexis_context_active_count", "Number of active conversation contexts").unwrap();

    /// Context window utilization percentage
    pub static ref CONTEXT_WINDOW_UTILIZATION: Histogram = register_histogram!(
        "nexis_context_window_utilization_percent",
        "Context window utilization (0-100%)",
        vec![10.0, 25.0, 50.0, 75.0, 90.0, 95.0, 99.0]
    ).unwrap();

    /// Messages per context
    pub static ref CONTEXT_MESSAGE_COUNT: Histogram = register_histogram!(
        "nexis_context_message_count",
        "Number of messages in contexts",
        vec![5.0, 10.0, 20.0, 50.0, 100.0, 200.0, 500.0]
    ).unwrap();

    // ============================================================================
    // Overflow Metrics
    // ============================================================================

    /// Window overflow events by strategy
    pub static ref OVERFLOW_EVENTS: CounterVec =
        register_counter_vec!("nexis_context_overflow_total", "Context window overflow events", &["strategy"]).unwrap();

    /// Messages truncated
    pub static ref MESSAGES_TRUNCATED: Gauge =
        register_gauge!("nexis_context_messages_truncated_total", "Total messages truncated").unwrap();

    // ============================================================================
    // Summarization Metrics
    // ============================================================================

    /// Summarization attempts
    pub static ref SUMMARIZATION_ATTEMPTS: CounterVec =
        register_counter_vec!("nexis_context_summarization_attempts", "Summarization attempts", &["status"]).unwrap();

    /// Summarization latency
    pub static ref SUMMARIZATION_LATENCY: Histogram = register_histogram!(
        "nexis_context_summarization_latency_seconds",
        "Summarization latency in seconds",
        vec![0.1, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0]
    ).unwrap();

    /// Messages summarized per operation
    pub static ref MESSAGES_SUMMARIZED: Histogram = register_histogram!(
        "nexis_context_messages_summarized",
        "Number of messages summarized per operation",
        vec![5.0, 10.0, 20.0, 50.0, 100.0]
    ).unwrap();

    /// Token savings from summarization
    pub static ref TOKEN_SAVINGS: Gauge =
        register_gauge!("nexis_context_token_savings_total", "Total tokens saved through summarization").unwrap();
}

/// Record a summarization success
pub fn record_summarization_success(messages_count: usize, latency_secs: f64) {
    SUMMARIZATION_ATTEMPTS.with_label_values(&["success"]).inc();
    SUMMARIZATION_LATENCY.observe(latency_secs);
    MESSAGES_SUMMARIZED.observe(messages_count as f64);
}

/// Record a summarization failure
pub fn record_summarization_failure() {
    SUMMARIZATION_ATTEMPTS.with_label_values(&["failure"]).inc();
}

/// Record a truncation event
pub fn record_truncation(messages_count: usize) {
    OVERFLOW_EVENTS.with_label_values(&["truncate"]).inc();
    MESSAGES_TRUNCATED.add(messages_count as f64);
}

/// Record a summarization overflow event
pub fn record_summarization_overflow() {
    OVERFLOW_EVENTS.with_label_values(&["summarize"]).inc();
}

/// Update active context count
pub fn set_active_contexts(count: usize) {
    CONTEXTS_ACTIVE.set(count as f64);
}

/// Record window utilization
pub fn record_window_utilization(utilization_percent: f64) {
    CONTEXT_WINDOW_UTILIZATION.observe(utilization_percent);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_are_registered() {
        // Just verify they don't panic
        set_active_contexts(5);
        record_window_utilization(75.0);
        record_truncation(3);
        record_summarization_overflow();
        record_summarization_success(10, 1.5);
        record_summarization_failure();
    }
}
