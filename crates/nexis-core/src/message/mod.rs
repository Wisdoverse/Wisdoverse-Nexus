//! Message domain extensions for Nexis.

pub use nexis_protocol::{Message, MessageContent};

#[derive(Debug, Clone)]
pub struct MessageBuilder {
    id: String,
    room_id: String,
    sender: nexis_protocol::MemberId,
    content: MessageContent,
    reply_to: Option<String>,
}

impl MessageBuilder {
    pub fn new(
        id: String,
        room_id: String,
        sender: nexis_protocol::MemberId,
        content: MessageContent,
    ) -> Self {
        Self {
            id,
            room_id,
            sender,
            content,
            reply_to: None,
        }
    }

    pub fn with_reply_to(mut self, reply_to: String) -> Self {
        self.reply_to = Some(reply_to);
        self
    }

    pub fn reply_to(&self) -> Option<&str> {
        self.reply_to.as_deref()
    }

    pub fn build(self) -> Message {
        Message {
            protocol_version: nexis_protocol::PROTOCOL_VERSION.to_string(),
            id: self.id,
            room_id: self.room_id,
            sender: self.sender,
            content: self.content,
            metadata: None,
            reply_to: self.reply_to,
            created_at: chrono::Utc::now(),
            updated_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexis_protocol::{MemberId, MemberType, MessageContent};

    fn make_sender() -> MemberId {
        MemberId::new(MemberType::Human, "alice").unwrap()
    }

    #[test]
    fn message_builder_basic() {
        let msg = MessageBuilder::new(
            "msg_1".into(),
            "room_1".into(),
            make_sender(),
            MessageContent::Text { text: "hi".into() },
        )
        .build();

        assert_eq!(msg.id, "msg_1");
        assert_eq!(msg.room_id, "room_1");
        assert!(msg.reply_to.is_none());
        assert!(msg.metadata.is_none());
        assert_eq!(msg.protocol_version, nexis_protocol::PROTOCOL_VERSION);
    }

    #[test]
    fn message_builder_with_reply_to() {
        let msg = MessageBuilder::new(
            "msg_2".into(),
            "room_1".into(),
            make_sender(),
            MessageContent::Text {
                text: "reply".into(),
            },
        )
        .with_reply_to("msg_1".into())
        .build();

        assert_eq!(msg.reply_to.as_deref(), Some("msg_1"));
    }

    #[test]
    fn message_builder_reply_to_accessor_none() {
        let builder = MessageBuilder::new(
            "msg_3".into(),
            "room_1".into(),
            make_sender(),
            MessageContent::Text {
                text: "test".into(),
            },
        );
        assert!(builder.reply_to().is_none());
    }

    #[test]
    fn message_builder_reply_to_accessor_some() {
        let builder = MessageBuilder::new(
            "msg_3".into(),
            "room_1".into(),
            make_sender(),
            MessageContent::Text {
                text: "test".into(),
            },
        )
        .with_reply_to("msg_1".into());
        assert_eq!(builder.reply_to(), Some("msg_1"));
    }

    #[test]
    fn built_message_validate_ok() {
        let msg = MessageBuilder::new(
            "msg_v".into(),
            "room_v".into(),
            make_sender(),
            MessageContent::Text { text: "ok".into() },
        )
        .build();
        assert!(msg.validate().is_ok());
    }

    #[test]
    fn built_message_validate_empty_room() {
        let msg = MessageBuilder::new(
            "msg_v".into(),
            "".into(),
            make_sender(),
            MessageContent::Text { text: "ok".into() },
        )
        .build();
        assert!(msg.validate().is_err());
    }
}
