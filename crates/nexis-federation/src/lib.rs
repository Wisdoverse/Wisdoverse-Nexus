//! Federation protocol primitives for Nexis.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::Sha256;
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

/// A unique identifier for an active federation connection.
pub type ConnectionId = u64;

/// Errors produced by federation protocol operations.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum FederationError {
    #[error("domain `{0}` is not in trust whitelist")]
    UntrustedDomain(String),
    #[error("missing verification key for domain `{0}`")]
    MissingVerificationKey(String),
    #[error("invalid handshake signature")]
    InvalidSignature,
    #[error("unknown remote domain `{0}`")]
    UnknownRemoteDomain(String),
    #[error("federation event delivery failed: {0}")]
    EventDeliveryFailed(String),
    #[error("rate limit exceeded for domain `{0}`")]
    RateLimitExceeded(String),
    #[error("domain `{0}` is temporarily blocked")]
    DomainBlocked(String),
}

/// Request payload used in server-to-server handshake.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeRequest {
    pub from_domain: String,
    pub to_domain: String,
    pub timestamp: DateTime<Utc>,
    pub nonce: String,
    pub signature: String,
}

impl HandshakeRequest {
    /// Creates an unsigned handshake request.
    pub fn new(
        from_domain: impl Into<String>,
        to_domain: impl Into<String>,
        nonce: impl Into<String>,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            from_domain: from_domain.into(),
            to_domain: to_domain.into(),
            timestamp,
            nonce: nonce.into(),
            signature: String::new(),
        }
    }

    /// Signs the request with an HMAC-SHA256 key.
    pub fn sign(&mut self, secret: &str) {
        let payload = self.canonical_payload();
        let mut mac =
            HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key length");
        mac.update(payload.as_bytes());
        self.signature = hex::encode(mac.finalize().into_bytes());
    }

    fn canonical_payload(&self) -> String {
        format!(
            "{}|{}|{}|{}",
            self.from_domain,
            self.to_domain,
            self.timestamp.timestamp_millis(),
            self.nonce
        )
    }
}

/// Established session metadata for a trusted peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeSession {
    pub peer_domain: String,
    pub connected_to: String,
    pub established_at: DateTime<Utc>,
}

/// Validates handshake requests against trust and key configuration.
pub struct HandshakeVerifier {
    trusted_domains: HashSet<String>,
    verification_keys: HashMap<String, String>,
}

impl HandshakeVerifier {
    /// Creates a handshake verifier from trusted domains and domain->key mappings.
    pub fn new<Domains, Keys, K1, K2>(trusted_domains: Domains, verification_keys: Keys) -> Self
    where
        Domains: IntoIterator,
        Domains::Item: AsRef<str>,
        Keys: IntoIterator<Item = (K1, K2)>,
        K1: AsRef<str>,
        K2: AsRef<str>,
    {
        Self {
            trusted_domains: trusted_domains
                .into_iter()
                .map(|domain| domain.as_ref().to_string())
                .collect(),
            verification_keys: verification_keys
                .into_iter()
                .map(|(domain, key)| (domain.as_ref().to_string(), key.as_ref().to_string()))
                .collect(),
        }
    }

    /// Creates a verifier from trust manager state (whitelist + active keys).
    pub fn from_trust_manager(trust_manager: &TrustManager) -> Self {
        Self::new(
            trust_manager.whitelist(),
            trust_manager.active_verification_keys(),
        )
    }

    /// Verifies signature and trust whitelist before accepting a handshake.
    pub fn verify(&self, request: &HandshakeRequest) -> Result<HandshakeSession, FederationError> {
        if !self.trusted_domains.contains(&request.from_domain) {
            return Err(FederationError::UntrustedDomain(
                request.from_domain.clone(),
            ));
        }

        let Some(key) = self.verification_keys.get(&request.from_domain) else {
            return Err(FederationError::MissingVerificationKey(
                request.from_domain.clone(),
            ));
        };

        let payload = request.canonical_payload();
        let mut mac =
            HmacSha256::new_from_slice(key.as_bytes()).expect("HMAC accepts any key length");
        mac.update(payload.as_bytes());
        let expected_signature = hex::encode(mac.finalize().into_bytes());
        if expected_signature != request.signature {
            return Err(FederationError::InvalidSignature);
        }

        Ok(HandshakeSession {
            peer_domain: request.from_domain.clone(),
            connected_to: request.to_domain.clone(),
            established_at: Utc::now(),
        })
    }
}

/// Tracks active federation sessions.
#[derive(Default)]
pub struct ConnectionManager {
    sessions: Arc<Mutex<HashMap<ConnectionId, HandshakeSession>>>,
    domain_index: Arc<Mutex<HashMap<String, ConnectionId>>>,
    next_id: AtomicU64,
}

impl ConnectionManager {
    /// Adds an established session and returns a connection identifier.
    pub fn add_session(&self, session: HandshakeSession) -> ConnectionId {
        let connection_id = self.next_id.fetch_add(1, Ordering::Relaxed) + 1;
        let peer_domain = session.peer_domain.clone();
        self.sessions
            .lock()
            .expect("sessions mutex poisoned")
            .insert(connection_id, session);
        self.domain_index
            .lock()
            .expect("domain_index mutex poisoned")
            .insert(peer_domain, connection_id);
        connection_id
    }

    /// Fetches session by connection identifier.
    pub fn get(&self, connection_id: ConnectionId) -> Option<HandshakeSession> {
        self.sessions
            .lock()
            .expect("sessions mutex poisoned")
            .get(&connection_id)
            .cloned()
    }

    /// Fetches a session by peer domain.
    pub fn get_by_domain(&self, domain: &str) -> Option<HandshakeSession> {
        let connection_id = self
            .domain_index
            .lock()
            .expect("domain_index mutex poisoned")
            .get(domain)
            .copied()?;
        self.get(connection_id)
    }

    /// Removes a session.
    pub fn remove(&self, connection_id: ConnectionId) -> Option<HandshakeSession> {
        let removed = self
            .sessions
            .lock()
            .expect("sessions mutex poisoned")
            .remove(&connection_id);
        if let Some(session) = &removed {
            self.domain_index
                .lock()
                .expect("domain_index mutex poisoned")
                .remove(&session.peer_domain);
        }
        removed
    }
}

/// A remote room discovered on another federated server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RemoteRoom {
    pub remote_domain: String,
    pub room_id: String,
}

/// Payload forwarded to a remote server in federation mode.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FederationMessage {
    pub room_id: String,
    pub event_id: String,
    pub payload: String,
}

impl FederationMessage {
    pub fn new(
        room_id: impl Into<String>,
        event_id: impl Into<String>,
        payload: impl Into<String>,
    ) -> Self {
        Self {
            room_id: room_id.into(),
            event_id: event_id.into(),
            payload: payload.into(),
        }
    }
}

/// Event kinds produced by federation links.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FederationEventKind {
    MessageForward,
    RoomDiscovered,
    HandshakeAccepted,
}

/// Versioned event model for cross-server federation traffic.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FederationEvent {
    pub event_id: String,
    pub source_domain: String,
    pub target_domain: String,
    pub kind: FederationEventKind,
    pub payload: Value,
    pub idempotency_key: String,
    pub occurred_at: DateTime<Utc>,
}

impl FederationEvent {
    pub fn new(
        event_id: impl Into<String>,
        source_domain: impl Into<String>,
        target_domain: impl Into<String>,
        kind: FederationEventKind,
        payload: Value,
    ) -> Self {
        let event_id = event_id.into();
        let source_domain = source_domain.into();
        let target_domain = target_domain.into();
        let occurred_at = Utc::now();
        let idempotency_key = build_idempotency_key(
            &event_id,
            &source_domain,
            &target_domain,
            &kind,
            occurred_at.timestamp_millis(),
        );
        Self {
            event_id,
            source_domain,
            target_domain,
            kind,
            payload,
            idempotency_key,
            occurred_at,
        }
    }
}

fn build_idempotency_key(
    event_id: &str,
    source_domain: &str,
    target_domain: &str,
    kind: &FederationEventKind,
    ts_ms: i64,
) -> String {
    let raw = format!("{event_id}:{source_domain}:{target_domain}:{kind:?}:{ts_ms}",);
    let mut hasher = Sha256::new();
    use sha2::Digest;
    hasher.update(raw.as_bytes());
    hex::encode(hasher.finalize())
}

/// Sliding replay window for event deduplication.
#[derive(Clone)]
pub struct ReplayWindow {
    window: ChronoDuration,
    seen: Arc<Mutex<HashMap<String, DateTime<Utc>>>>,
}

impl ReplayWindow {
    pub fn new(window_seconds: i64) -> Self {
        Self {
            window: ChronoDuration::seconds(window_seconds.max(1)),
            seen: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn should_accept(&self, event: &FederationEvent) -> bool {
        self.accept_key(&event.idempotency_key, event.occurred_at)
    }

    fn accept_key(&self, key: &str, occurred_at: DateTime<Utc>) -> bool {
        let mut seen = self.seen.lock().expect("replay window mutex poisoned");
        let cutoff = Utc::now() - self.window;
        seen.retain(|_, ts| *ts >= cutoff);

        if seen.contains_key(key) {
            return false;
        }
        seen.insert(key.to_string(), occurred_at);
        true
    }
}

/// Dead-letter event entry after exhausting retries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterEntry {
    pub event: FederationEvent,
    pub failed_at: DateTime<Utc>,
    pub attempts: u32,
    pub reason: String,
}

/// Bounded dead-letter queue.
#[derive(Clone)]
pub struct DeadLetterQueue {
    capacity: usize,
    entries: Arc<Mutex<VecDeque<DeadLetterEntry>>>,
}

impl DeadLetterQueue {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            entries: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn push(&self, entry: DeadLetterEntry) {
        let mut entries = self
            .entries
            .lock()
            .expect("dead letter queue mutex poisoned");
        entries.push_back(entry);
        while entries.len() > self.capacity {
            let _ = entries.pop_front();
        }
    }

    pub fn list(&self) -> Vec<DeadLetterEntry> {
        self.entries
            .lock()
            .expect("dead letter queue mutex poisoned")
            .iter()
            .cloned()
            .collect()
    }
}

/// Snapshot metrics for federation event processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventMetricsSnapshot {
    pub processed: u64,
    pub deduplicated: u64,
    pub retried: u64,
    pub dead_lettered: u64,
}

#[derive(Default)]
struct EventMetrics {
    processed: AtomicU64,
    deduplicated: AtomicU64,
    retried: AtomicU64,
    dead_lettered: AtomicU64,
}

impl EventMetrics {
    fn snapshot(&self) -> EventMetricsSnapshot {
        EventMetricsSnapshot {
            processed: self.processed.load(Ordering::Relaxed),
            deduplicated: self.deduplicated.load(Ordering::Relaxed),
            retried: self.retried.load(Ordering::Relaxed),
            dead_lettered: self.dead_lettered.load(Ordering::Relaxed),
        }
    }
}

/// Federation event processor with retry, replay window and dead-letter handling.
pub struct FederationEventProcessor {
    max_retries: u32,
    replay_window: ReplayWindow,
    dead_letters: DeadLetterQueue,
    metrics: Arc<EventMetrics>,
}

#[derive(Debug, Clone)]
struct KeyRing {
    active_key_id: String,
    keys: HashMap<String, String>,
}

/// Trust-domain whitelist and key-rotation state.
#[derive(Default)]
pub struct TrustManager {
    trusted_domains: Arc<Mutex<HashSet<String>>>,
    key_rings: Arc<Mutex<HashMap<String, KeyRing>>>,
}

impl TrustManager {
    pub fn new<Domains, Keys, D, KId, S>(trusted_domains: Domains, initial_keys: Keys) -> Self
    where
        Domains: IntoIterator<Item = D>,
        D: AsRef<str>,
        Keys: IntoIterator<Item = (D, KId, S)>,
        KId: AsRef<str>,
        S: AsRef<str>,
    {
        let manager = Self {
            trusted_domains: Arc::new(Mutex::new(
                trusted_domains
                    .into_iter()
                    .map(|domain| domain.as_ref().to_string())
                    .collect(),
            )),
            key_rings: Arc::new(Mutex::new(HashMap::new())),
        };

        {
            let mut rings = manager.key_rings.lock().expect("key_rings mutex poisoned");
            for (domain, key_id, secret) in initial_keys {
                let domain = domain.as_ref().to_string();
                let key_id = key_id.as_ref().to_string();
                let secret = secret.as_ref().to_string();
                let ring = rings.entry(domain).or_insert(KeyRing {
                    active_key_id: key_id.clone(),
                    keys: HashMap::new(),
                });
                ring.keys.insert(key_id.clone(), secret);
                ring.active_key_id = key_id;
            }
        }

        manager
    }

    pub fn is_trusted(&self, domain: &str) -> bool {
        self.trusted_domains
            .lock()
            .expect("trusted_domains mutex poisoned")
            .contains(domain)
    }

    pub fn whitelist(&self) -> Vec<String> {
        self.trusted_domains
            .lock()
            .expect("trusted_domains mutex poisoned")
            .iter()
            .cloned()
            .collect()
    }

    pub fn active_verification_keys(&self) -> Vec<(String, String)> {
        self.key_rings
            .lock()
            .expect("key_rings mutex poisoned")
            .iter()
            .filter_map(|(domain, ring)| {
                ring.keys
                    .get(&ring.active_key_id)
                    .map(|secret| (domain.clone(), secret.clone()))
            })
            .collect()
    }

    pub fn rotate_key(
        &self,
        domain: &str,
        new_key_id: &str,
        new_secret: &str,
    ) -> Result<(), FederationError> {
        if !self.is_trusted(domain) {
            return Err(FederationError::UntrustedDomain(domain.to_string()));
        }

        let mut rings = self.key_rings.lock().expect("key_rings mutex poisoned");
        let ring = rings.entry(domain.to_string()).or_insert(KeyRing {
            active_key_id: new_key_id.to_string(),
            keys: HashMap::new(),
        });
        ring.keys
            .insert(new_key_id.to_string(), new_secret.to_string());
        ring.active_key_id = new_key_id.to_string();
        Ok(())
    }
}

/// Fixed-window rate limiter keyed by remote domain.
#[derive(Clone)]
pub struct RateLimiter {
    max_requests: usize,
    window: ChronoDuration,
    buckets: Arc<Mutex<HashMap<String, VecDeque<DateTime<Utc>>>>>,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_secs: i64) -> Self {
        Self {
            max_requests: max_requests.max(1),
            window: ChronoDuration::seconds(window_secs.max(1)),
            buckets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn check(&self, key: &str) -> bool {
        let now = Utc::now();
        let cutoff = now - self.window;
        let mut buckets = self.buckets.lock().expect("rate limiter mutex poisoned");
        let bucket = buckets.entry(key.to_string()).or_default();
        while bucket.front().is_some_and(|ts| *ts < cutoff) {
            let _ = bucket.pop_front();
        }
        if bucket.len() >= self.max_requests {
            return false;
        }
        bucket.push_back(now);
        true
    }
}

#[derive(Debug, Clone)]
struct AbuseState {
    failures: u32,
    blocked_until: Option<DateTime<Utc>>,
}

/// Detects repeated authentication failures and temporarily blocks abusive domains.
#[derive(Clone)]
pub struct AbuseDetector {
    threshold: u32,
    block_for: ChronoDuration,
    states: Arc<Mutex<HashMap<String, AbuseState>>>,
}

impl AbuseDetector {
    pub fn new(threshold: u32, block_secs: i64) -> Self {
        Self {
            threshold: threshold.max(1),
            block_for: ChronoDuration::seconds(block_secs.max(1)),
            states: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn record_failure(&self, domain: &str) {
        let now = Utc::now();
        let mut states = self.states.lock().expect("abuse detector mutex poisoned");
        let state = states.entry(domain.to_string()).or_insert(AbuseState {
            failures: 0,
            blocked_until: None,
        });
        state.failures += 1;
        if state.failures >= self.threshold {
            state.blocked_until = Some(now + self.block_for);
        }
    }

    pub fn record_success(&self, domain: &str) {
        self.states
            .lock()
            .expect("abuse detector mutex poisoned")
            .insert(
                domain.to_string(),
                AbuseState {
                    failures: 0,
                    blocked_until: None,
                },
            );
    }

    pub fn is_blocked(&self, domain: &str) -> bool {
        let now = Utc::now();
        let mut states = self.states.lock().expect("abuse detector mutex poisoned");
        if let Some(state) = states.get_mut(domain) {
            if let Some(blocked_until) = state.blocked_until {
                if blocked_until > now {
                    return true;
                }
                state.blocked_until = None;
                state.failures = 0;
            }
        }
        false
    }
}

/// Combined T11 security policy used at federation request ingress.
#[derive(Clone)]
pub struct FederationSecurityPolicy {
    trust_manager: Arc<TrustManager>,
    rate_limiter: RateLimiter,
    abuse_detector: AbuseDetector,
}

impl FederationSecurityPolicy {
    pub fn new(
        trust_manager: Arc<TrustManager>,
        max_requests: usize,
        window_secs: i64,
        abuse_threshold: u32,
        block_secs: i64,
    ) -> Self {
        Self {
            trust_manager,
            rate_limiter: RateLimiter::new(max_requests, window_secs),
            abuse_detector: AbuseDetector::new(abuse_threshold, block_secs),
        }
    }

    pub fn preflight(&self, domain: &str) -> Result<(), FederationError> {
        if !self.trust_manager.is_trusted(domain) {
            return Err(FederationError::UntrustedDomain(domain.to_string()));
        }
        if self.abuse_detector.is_blocked(domain) {
            return Err(FederationError::DomainBlocked(domain.to_string()));
        }
        if !self.rate_limiter.check(domain) {
            return Err(FederationError::RateLimitExceeded(domain.to_string()));
        }
        Ok(())
    }

    pub fn record_handshake_result(&self, domain: &str, success: bool) {
        if success {
            self.abuse_detector.record_success(domain);
        } else {
            self.abuse_detector.record_failure(domain);
        }
    }
}

impl FederationEventProcessor {
    pub fn new(max_retries: u32, replay_window_secs: i64) -> Self {
        Self {
            max_retries,
            replay_window: ReplayWindow::new(replay_window_secs),
            dead_letters: DeadLetterQueue::new(1000),
            metrics: Arc::new(EventMetrics::default()),
        }
    }

    pub async fn process_with_retry<F, Fut, E>(
        &self,
        event: FederationEvent,
        mut handler: F,
    ) -> Result<(), FederationError>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<(), E>>,
        E: std::fmt::Display,
    {
        if !self.replay_window.should_accept(&event) {
            self.metrics.deduplicated.fetch_add(1, Ordering::Relaxed);
            return Ok(());
        }

        let mut attempts = 0_u32;
        loop {
            attempts += 1;
            match handler().await {
                Ok(()) => {
                    self.metrics.processed.fetch_add(1, Ordering::Relaxed);
                    return Ok(());
                }
                Err(_error) if attempts <= self.max_retries => {
                    self.metrics.retried.fetch_add(1, Ordering::Relaxed);
                    continue;
                }
                Err(error) => {
                    let reason = error.to_string();
                    self.dead_letters.push(DeadLetterEntry {
                        event,
                        failed_at: Utc::now(),
                        attempts,
                        reason: reason.clone(),
                    });
                    self.metrics.dead_lettered.fetch_add(1, Ordering::Relaxed);
                    return Err(FederationError::EventDeliveryFailed(reason));
                }
            }
        }
    }

    pub fn dead_letter_queue(&self) -> Vec<DeadLetterEntry> {
        self.dead_letters.list()
    }

    pub fn metrics(&self) -> EventMetricsSnapshot {
        self.metrics.snapshot()
    }
}

/// Federation client used for remote room discovery and message forwarding.
#[async_trait]
pub trait FederationClient: Send + Sync {
    async fn discover_rooms(&self, remote_domain: &str)
        -> Result<Vec<RemoteRoom>, FederationError>;

    async fn forward_message(
        &self,
        remote_domain: &str,
        message: FederationMessage,
    ) -> Result<(), FederationError>;
}

/// In-memory federation client for local testing and simulation.
#[derive(Default)]
pub struct InMemoryFederationClient {
    rooms: Arc<Mutex<HashMap<String, Vec<RemoteRoom>>>>,
    forwarded_messages: Arc<Mutex<Vec<(String, FederationMessage)>>>,
}

impl InMemoryFederationClient {
    pub fn with_rooms<I>(domain: &str, rooms: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        let mapped_rooms = rooms
            .into_iter()
            .map(|room_id| RemoteRoom {
                remote_domain: domain.to_string(),
                room_id,
            })
            .collect();
        let mut table = HashMap::new();
        table.insert(domain.to_string(), mapped_rooms);
        Self {
            rooms: Arc::new(Mutex::new(table)),
            forwarded_messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn forwarded(&self) -> Vec<(String, FederationMessage)> {
        self.forwarded_messages
            .lock()
            .expect("forwarded_messages mutex poisoned")
            .clone()
    }
}

#[async_trait]
impl FederationClient for InMemoryFederationClient {
    async fn discover_rooms(
        &self,
        remote_domain: &str,
    ) -> Result<Vec<RemoteRoom>, FederationError> {
        let Some(rooms) = self
            .rooms
            .lock()
            .expect("rooms mutex poisoned")
            .get(remote_domain)
            .cloned()
        else {
            return Err(FederationError::UnknownRemoteDomain(
                remote_domain.to_string(),
            ));
        };
        Ok(rooms)
    }

    async fn forward_message(
        &self,
        remote_domain: &str,
        message: FederationMessage,
    ) -> Result<(), FederationError> {
        if !self
            .rooms
            .lock()
            .expect("rooms mutex poisoned")
            .contains_key(remote_domain)
        {
            return Err(FederationError::UnknownRemoteDomain(
                remote_domain.to_string(),
            ));
        }
        self.forwarded_messages
            .lock()
            .expect("forwarded_messages mutex poisoned")
            .push((remote_domain.to_string(), message));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;

    use crate::{
        AbuseDetector, ConnectionManager, FederationClient, FederationEvent, FederationEventKind,
        FederationEventProcessor, FederationMessage, HandshakeRequest, HandshakeVerifier,
        InMemoryFederationClient, RateLimiter, ReplayWindow, TrustManager,
    };

    #[tokio::test]
    async fn handshake_accepts_trusted_domain_with_valid_signature() {
        let verifier = HandshakeVerifier::new(["alpha.example"], [("alpha.example", "secret-key")]);
        let mut req = HandshakeRequest::new("alpha.example", "beta.example", "nonce-1", Utc::now());
        req.sign("secret-key");

        let session = verifier.verify(&req).expect("handshake should succeed");
        assert_eq!(session.peer_domain, "alpha.example");
    }

    #[tokio::test]
    async fn handshake_rejects_untrusted_domain() {
        let verifier = HandshakeVerifier::new(["alpha.example"], [("alpha.example", "secret-key")]);
        let mut req = HandshakeRequest::new("evil.example", "beta.example", "nonce-1", Utc::now());
        req.sign("secret-key");

        assert!(verifier.verify(&req).is_err());
    }

    #[tokio::test]
    async fn handshake_rejects_bad_signature() {
        let verifier = HandshakeVerifier::new(["alpha.example"], [("alpha.example", "secret-key")]);
        let mut req = HandshakeRequest::new("alpha.example", "beta.example", "nonce-1", Utc::now());
        req.sign("wrong-key");

        assert!(verifier.verify(&req).is_err());
    }

    #[tokio::test]
    async fn connection_manager_tracks_sessions() {
        let verifier = HandshakeVerifier::new(["alpha.example"], [("alpha.example", "secret-key")]);
        let mut req = HandshakeRequest::new("alpha.example", "beta.example", "nonce-1", Utc::now());
        req.sign("secret-key");

        let session = verifier.verify(&req).expect("handshake should succeed");
        let manager = ConnectionManager::default();
        let connection_id = manager.add_session(session.clone());

        assert!(manager.get(connection_id).is_some());
        assert!(manager.get_by_domain("alpha.example").is_some());

        manager.remove(connection_id);
        assert!(manager.get(connection_id).is_none());
    }

    #[tokio::test]
    async fn in_memory_client_discovers_rooms_and_forwards_message() {
        let client = InMemoryFederationClient::with_rooms(
            "alpha.example",
            ["room-1".to_string(), "room-2".to_string()],
        );

        let rooms = client
            .discover_rooms("alpha.example")
            .await
            .expect("discover should succeed");
        assert_eq!(rooms.len(), 2);

        let msg = FederationMessage::new("room-1", "evt-1", "hello");
        client
            .forward_message("alpha.example", msg)
            .await
            .expect("forward should succeed");

        assert_eq!(client.forwarded().len(), 1);
    }

    #[test]
    fn federation_event_has_idempotency_key() {
        let event = FederationEvent::new(
            "evt-1",
            "alpha.example",
            "beta.example",
            FederationEventKind::MessageForward,
            json!({"roomId": "room-1"}),
        );
        assert!(!event.idempotency_key.is_empty());
    }

    #[test]
    fn replay_window_deduplicates_within_window() {
        let window = ReplayWindow::new(300);
        let event = FederationEvent::new(
            "evt-2",
            "alpha.example",
            "beta.example",
            FederationEventKind::MessageForward,
            json!({"roomId": "room-1"}),
        );

        assert!(window.should_accept(&event));
        assert!(!window.should_accept(&event));
    }

    #[tokio::test]
    async fn processor_retries_and_moves_to_dead_letter() {
        let processor = FederationEventProcessor::new(2, 300);
        let event = FederationEvent::new(
            "evt-3",
            "alpha.example",
            "beta.example",
            FederationEventKind::MessageForward,
            json!({"roomId": "room-1"}),
        );

        let result = processor
            .process_with_retry(event, || async { Err::<(), _>("boom") })
            .await;
        assert!(result.is_err());
        assert_eq!(processor.dead_letter_queue().len(), 1);
        assert_eq!(processor.metrics().dead_lettered, 1);
    }

    #[test]
    fn trust_manager_rotates_active_key() {
        let trust = TrustManager::new(["alpha.example"], [("alpha.example", "k1", "secret-v1")]);
        let verifier = HandshakeVerifier::from_trust_manager(&trust);
        let mut old_req =
            HandshakeRequest::new("alpha.example", "beta.example", "nonce-1", Utc::now());
        old_req.sign("secret-v1");
        assert!(verifier.verify(&old_req).is_ok());

        trust
            .rotate_key("alpha.example", "k2", "secret-v2")
            .expect("rotation should succeed");
        let verifier_after = HandshakeVerifier::from_trust_manager(&trust);
        assert!(verifier_after.verify(&old_req).is_err());

        let mut new_req =
            HandshakeRequest::new("alpha.example", "beta.example", "nonce-2", Utc::now());
        new_req.sign("secret-v2");
        assert!(verifier_after.verify(&new_req).is_ok());
    }

    #[test]
    fn rate_limiter_rejects_excess_requests() {
        let limiter = RateLimiter::new(2, 60);
        assert!(limiter.check("alpha.example"));
        assert!(limiter.check("alpha.example"));
        assert!(!limiter.check("alpha.example"));
    }

    #[test]
    fn abuse_detector_blocks_after_threshold() {
        let detector = AbuseDetector::new(3, 300);
        assert!(!detector.is_blocked("alpha.example"));
        detector.record_failure("alpha.example");
        detector.record_failure("alpha.example");
        detector.record_failure("alpha.example");
        assert!(detector.is_blocked("alpha.example"));
    }
}
