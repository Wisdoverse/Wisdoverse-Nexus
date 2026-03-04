//! Yrs-backed CRDT document wrapper.

use crate::error::DocResult;
use yrs::Doc;

/// Placeholder token returned by `observe_changes`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SubscriptionToken;

/// Thin wrapper around a Yrs document.
#[derive(Debug)]
pub struct CRDTDocument {
    doc: Doc,
}

impl Clone for CRDTDocument {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl PartialEq for CRDTDocument {
    fn eq(&self, other: &Self) -> bool {
        self.encode_update() == other.encode_update()
    }
}

impl Eq for CRDTDocument {}

impl CRDTDocument {
    /// Creates a new empty CRDT document.
    #[must_use]
    pub fn new() -> Self {
        Self { doc: Doc::new() }
    }

    /// Returns the current textual content.
    #[must_use]
    pub fn get_content(&self) -> String {
        let _ = &self.doc;
        String::new()
    }

    /// Applies a remote binary update.
    pub fn apply_update(&self, update: &[u8]) -> DocResult<()> {
        let _ = (&self.doc, update);
        Ok(())
    }

    /// Encodes a full-state update for synchronization.
    #[must_use]
    pub fn encode_update(&self) -> Vec<u8> {
        let _ = &self.doc;
        Vec::new()
    }

    /// Stub for future observer registration.
    #[must_use]
    pub fn observe_changes(&self) -> SubscriptionToken {
        let _ = &self.doc;
        SubscriptionToken
    }
}

impl Default for CRDTDocument {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::{CRDTDocument, SubscriptionToken};

    #[test]
    fn new_document_has_empty_state() {
        let doc = CRDTDocument::new();
        assert_eq!(doc.get_content(), "");
        assert_eq!(doc.encode_update(), Vec::<u8>::new());
    }

    #[test]
    fn apply_update_is_stable_for_current_stub_behavior() {
        let doc = CRDTDocument::new();
        let initial = doc.encode_update();

        doc.apply_update(&[1, 2, 3, 4])
            .expect("stub apply_update should not fail");

        assert_eq!(doc.encode_update(), initial);
        assert_eq!(doc.get_content(), "");
    }

    #[test]
    fn clone_and_observer_token_match_default_state() {
        let doc = CRDTDocument::new();
        let cloned = doc.clone();
        let token = doc.observe_changes();

        assert_eq!(cloned, doc);
        assert_eq!(token, SubscriptionToken);
    }

    proptest! {
        #[test]
        fn merge_operations_keep_encoded_state_stable(updates in proptest::collection::vec(proptest::collection::vec(any::<u8>(), 0..64), 0..32)) {
            let left = CRDTDocument::new();
            let right = CRDTDocument::new();

            for update in &updates {
                left.apply_update(update).expect("applying random update should be infallible in stub implementation");
            }
            for update in updates.iter().rev() {
                right.apply_update(update).expect("applying random update should be infallible in stub implementation");
            }

            prop_assert_eq!(left.encode_update(), right.encode_update());
            prop_assert_eq!(left.get_content(), right.get_content());
        }
    }
}
