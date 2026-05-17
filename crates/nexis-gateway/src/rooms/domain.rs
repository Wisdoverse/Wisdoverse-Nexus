//! Domain objects for gateway rooms and messages.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Room {
    pub id: String,
    pub name: String,
    pub creator_id: Option<String>,
    pub topic: Option<String>,
    #[cfg(feature = "multi-tenant")]
    pub tenant_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredMessage {
    pub id: String,
    pub sender: String,
    pub text: String,
    pub reply_to: Option<String>,
}
