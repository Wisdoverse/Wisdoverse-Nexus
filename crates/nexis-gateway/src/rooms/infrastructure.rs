//! Infrastructure adapters for room and message storage.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
#[cfg(feature = "persistence-sqlx")]
use sqlx::Row;
use tokio::sync::{RwLock, Semaphore};

use crate::crypto::DataEncryption;
#[cfg(feature = "persistence-sqlx")]
use crate::db::DatabasePool;

use super::application::{
    InviteMemberResult, ListRoomsResult, MemberDataExport, MemberDeletionResult,
    MemberMessageRecord, MemberRoomRecord, RoomApplication, RoomCommandError, RoomDetails,
    RoomRepository, RoomSummary,
};
use super::domain::{Room, StoredMessage};

#[derive(Clone)]
pub struct InMemoryRoomRepository {
    rooms: Arc<RwLock<HashMap<String, Room>>>,
    room_messages: Arc<RwLock<HashMap<String, Vec<StoredMessage>>>>,
    room_members: Arc<RwLock<HashMap<String, Vec<String>>>>,
    write_gate: Arc<Semaphore>,
    encryption: Option<DataEncryption>,
}

impl Default for InMemoryRoomRepository {
    fn default() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
            room_messages: Arc::new(RwLock::new(HashMap::new())),
            room_members: Arc::new(RwLock::new(HashMap::new())),
            write_gate: Arc::new(Semaphore::new(2_048)),
            encryption: DataEncryption::from_env(),
        }
    }
}

impl InMemoryRoomRepository {
    #[cfg(test)]
    pub fn without_encryption() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
            room_messages: Arc::new(RwLock::new(HashMap::new())),
            room_members: Arc::new(RwLock::new(HashMap::new())),
            write_gate: Arc::new(Semaphore::new(2_048)),
            encryption: None,
        }
    }

    async fn acquire_write(&self) -> Result<tokio::sync::OwnedSemaphorePermit, RoomCommandError> {
        self.write_gate
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| RoomCommandError::ServiceUnavailable)
    }
}

impl Default for RoomApplication {
    fn default() -> Self {
        Self::new(Arc::new(InMemoryRoomRepository::default()))
    }
}

#[async_trait]
impl RoomRepository for InMemoryRoomRepository {
    async fn active_room_count(&self) -> usize {
        self.rooms.read().await.len()
    }

    async fn create_room(&self, room: Room) -> Result<Room, RoomCommandError> {
        let _permit = self.acquire_write().await?;
        self.rooms
            .write()
            .await
            .insert(room.id.clone(), room.clone());
        Ok(room)
    }

    async fn append_message(
        &self,
        room_id: String,
        mut message: StoredMessage,
    ) -> Result<StoredMessage, RoomCommandError> {
        if !self.rooms.read().await.contains_key(&room_id) {
            return Err(RoomCommandError::RoomNotFound);
        }

        if let Some(ref enc) = self.encryption {
            message.text = enc.encrypt_string(&message.text).unwrap_or_else(|error| {
                tracing::error!("encryption failed: {}\n", error);
                message.text.clone()
            });
        }

        let _permit = self.acquire_write().await?;
        self.room_messages
            .write()
            .await
            .entry(room_id)
            .or_default()
            .push(message.clone());

        Ok(message)
    }

    async fn get_room(&self, id: &str) -> Result<RoomDetails, RoomCommandError> {
        let Some(room) = self.rooms.read().await.get(id).cloned() else {
            return Err(RoomCommandError::RoomNotFound);
        };

        let raw_messages = self
            .room_messages
            .read()
            .await
            .get(id)
            .cloned()
            .unwrap_or_default();

        let messages = if let Some(ref enc) = self.encryption {
            raw_messages
                .into_iter()
                .map(|mut msg| {
                    msg.text = enc.decrypt_string(&msg.text).unwrap_or_else(|error| {
                        tracing::warn!("failed to decrypt message {}: {}", msg.id, error);
                        msg.text.clone()
                    });
                    msg
                })
                .collect()
        } else {
            raw_messages
        };

        Ok(RoomDetails { room, messages })
    }

    async fn invite_member(
        &self,
        room_id: String,
        member_id: String,
    ) -> Result<InviteMemberResult, RoomCommandError> {
        if !self.rooms.read().await.contains_key(&room_id) {
            return Err(RoomCommandError::RoomNotFound);
        }

        let _permit = self.acquire_write().await?;
        let mut members = self.room_members.write().await;
        let room_members = members.entry(room_id.clone()).or_default();
        if !room_members.contains(&member_id) {
            room_members.push(member_id.clone());
        }

        Ok(InviteMemberResult { room_id, member_id })
    }

    async fn list_rooms(&self, limit: usize, offset: usize) -> ListRoomsResult {
        let rooms = self.rooms.read().await;
        let members = self.room_members.read().await;

        let items = rooms
            .values()
            .skip(offset)
            .take(limit)
            .map(|room| RoomSummary {
                id: room.id.clone(),
                name: room.name.clone(),
                topic: room.topic.clone(),
                member_count: members.get(&room.id).map(Vec::len),
            })
            .collect();

        ListRoomsResult {
            rooms: items,
            total: rooms.len(),
        }
    }

    async fn delete_room(&self, id: &str) -> Result<(), RoomCommandError> {
        let _permit = self.acquire_write().await?;

        if self.rooms.write().await.remove(id).is_none() {
            return Err(RoomCommandError::RoomNotFound);
        }

        self.room_messages.write().await.remove(id);
        self.room_members.write().await.remove(id);

        Ok(())
    }

    async fn export_member_data(&self, member_id: &str) -> MemberDataExport {
        let messages = {
            let room_messages = self.room_messages.read().await;
            room_messages
                .iter()
                .flat_map(|(room_id, list)| {
                    list.iter()
                        .filter(|message| message.sender == member_id)
                        .map(|message| MemberMessageRecord {
                            room_id: room_id.clone(),
                            message: message.clone(),
                        })
                })
                .collect()
        };

        let rooms = {
            let rooms = self.rooms.read().await;
            rooms
                .values()
                .filter(|room| room.creator_id.as_deref() == Some(member_id))
                .map(|room| MemberRoomRecord {
                    id: room.id.clone(),
                    name: room.name.clone(),
                    topic: room.topic.clone(),
                })
                .collect()
        };

        MemberDataExport { messages, rooms }
    }

    async fn delete_member_data(
        &self,
        member_id: &str,
    ) -> Result<MemberDeletionResult, RoomCommandError> {
        let _permit = self.acquire_write().await?;

        let mut message_count = 0i64;
        {
            let mut messages = self.room_messages.write().await;
            for list in messages.values_mut() {
                for message in list.iter_mut() {
                    if message.sender == member_id {
                        message.text = "[Content removed per GDPR request]".to_string();
                        message.sender = "[deleted]".to_string();
                        message_count += 1;
                    }
                }
            }
        }

        {
            let mut room_members = self.room_members.write().await;
            for members in room_members.values_mut() {
                if let Some(position) = members.iter().position(|member| member == member_id) {
                    members.remove(position);
                }
            }
        }

        let room_ids = {
            let rooms = self.rooms.read().await;
            rooms
                .iter()
                .filter(|(_, room)| room.creator_id.as_deref() == Some(member_id))
                .map(|(id, _)| id.clone())
                .collect::<Vec<_>>()
        };
        let rooms_created = room_ids.len() as i64;

        {
            let mut rooms = self.rooms.write().await;
            for id in &room_ids {
                rooms.remove(id);
            }
        }

        {
            let mut messages = self.room_messages.write().await;
            let mut members = self.room_members.write().await;
            for id in &room_ids {
                messages.remove(id);
                members.remove(id);
            }
        }

        Ok(MemberDeletionResult {
            messages: message_count,
            rooms_created,
        })
    }
}

#[cfg(feature = "persistence-sqlx")]
#[derive(Clone)]
pub struct SqlxRoomRepository {
    pool: DatabasePool,
    encryption: Option<DataEncryption>,
}

#[cfg(feature = "persistence-sqlx")]
impl SqlxRoomRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self {
            pool,
            encryption: DataEncryption::from_env(),
        }
    }

    #[cfg(test)]
    pub fn without_encryption(pool: DatabasePool) -> Self {
        Self {
            pool,
            encryption: None,
        }
    }

    async fn load_room(&self, id: &str) -> Result<Room, RoomCommandError> {
        let row =
            sqlx::query("SELECT id, name, creator_id, topic, tenant_id FROM rooms WHERE id = $1")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|error| {
                    tracing::error!("failed to load room {id}: {error}");
                    RoomCommandError::ServiceUnavailable
                })?
                .ok_or(RoomCommandError::RoomNotFound)?;

        Ok(Room {
            id: row.get("id"),
            name: row.get("name"),
            creator_id: row.try_get("creator_id").ok(),
            topic: row.try_get("topic").ok(),
            #[cfg(feature = "multi-tenant")]
            tenant_id: row.try_get("tenant_id").ok(),
        })
    }

    fn decrypt_message_text(&self, message_id: &str, text: String) -> String {
        let Some(encryption) = &self.encryption else {
            return text;
        };

        encryption.decrypt_string(&text).unwrap_or_else(|error| {
            tracing::warn!("failed to decrypt message {message_id}: {error}");
            text
        })
    }

    fn encrypt_message_text(&self, text: &str) -> String {
        let Some(encryption) = &self.encryption else {
            return text.to_string();
        };

        encryption.encrypt_string(text).unwrap_or_else(|error| {
            tracing::error!("encryption failed: {error}");
            text.to_string()
        })
    }
}

#[cfg(feature = "persistence-sqlx")]
#[async_trait]
impl RoomRepository for SqlxRoomRepository {
    async fn active_room_count(&self) -> usize {
        match sqlx::query("SELECT COUNT(*) AS count FROM rooms")
            .fetch_one(&self.pool)
            .await
        {
            Ok(row) => {
                let count: i64 = row.get("count");
                usize::try_from(count).unwrap_or(0)
            }
            Err(error) => {
                tracing::error!("failed to count active rooms: {error}");
                0
            }
        }
    }

    async fn create_room(&self, room: Room) -> Result<Room, RoomCommandError> {
        #[cfg(feature = "multi-tenant")]
        let query_result = sqlx::query(
            "INSERT INTO rooms (id, name, creator_id, topic, tenant_id) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&room.id)
        .bind(&room.name)
        .bind(&room.creator_id)
        .bind(&room.topic)
        .bind(&room.tenant_id)
        .execute(&self.pool)
        .await;

        #[cfg(not(feature = "multi-tenant"))]
        let query_result =
            sqlx::query("INSERT INTO rooms (id, name, creator_id, topic) VALUES ($1, $2, $3, $4)")
                .bind(&room.id)
                .bind(&room.name)
                .bind(&room.creator_id)
                .bind(&room.topic)
                .execute(&self.pool)
                .await;

        query_result.map_err(|error| {
            tracing::error!("failed to create room {}: {error}", room.id);
            RoomCommandError::ServiceUnavailable
        })?;

        Ok(room)
    }

    async fn append_message(
        &self,
        room_id: String,
        message: StoredMessage,
    ) -> Result<StoredMessage, RoomCommandError> {
        #[cfg(feature = "multi-tenant")]
        let tenant_id = self.load_room(&room_id).await?.tenant_id;
        #[cfg(not(feature = "multi-tenant"))]
        self.load_room(&room_id).await?;
        let stored_text = self.encrypt_message_text(&message.text);

        #[cfg(feature = "multi-tenant")]
        let query_result = sqlx::query(
            "INSERT INTO messages (id, room_id, sender_id, content, reply_to, tenant_id) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&message.id)
        .bind(&room_id)
        .bind(&message.sender)
        .bind(&stored_text)
        .bind(&message.reply_to)
        .bind(&tenant_id)
        .execute(&self.pool)
        .await;

        #[cfg(not(feature = "multi-tenant"))]
        let query_result = sqlx::query(
            "INSERT INTO messages (id, room_id, sender_id, content, reply_to) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&message.id)
        .bind(&room_id)
        .bind(&message.sender)
        .bind(&stored_text)
        .bind(&message.reply_to)
        .execute(&self.pool)
        .await;

        query_result.map_err(|error| {
            tracing::error!("failed to append message {}: {error}", message.id);
            RoomCommandError::ServiceUnavailable
        })?;

        Ok(message)
    }

    async fn get_room(&self, id: &str) -> Result<RoomDetails, RoomCommandError> {
        let room = self.load_room(id).await?;
        let rows = sqlx::query(
            "SELECT id, sender_id, content, reply_to FROM messages \
             WHERE room_id = $1 ORDER BY created_at ASC",
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            tracing::error!("failed to load messages for room {id}: {error}");
            RoomCommandError::ServiceUnavailable
        })?;

        let messages = rows
            .into_iter()
            .map(|row| {
                let id: String = row.get("id");
                let text: String = row.get("content");
                StoredMessage {
                    id: id.clone(),
                    sender: row.get("sender_id"),
                    text: self.decrypt_message_text(&id, text),
                    reply_to: row.try_get("reply_to").ok(),
                }
            })
            .collect();

        Ok(RoomDetails { room, messages })
    }

    async fn invite_member(
        &self,
        room_id: String,
        member_id: String,
    ) -> Result<InviteMemberResult, RoomCommandError> {
        self.load_room(&room_id).await?;
        sqlx::query(
            "INSERT INTO room_members (room_id, member_id) VALUES ($1, $2) \
             ON CONFLICT (room_id, member_id) DO NOTHING",
        )
        .bind(&room_id)
        .bind(&member_id)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            tracing::error!("failed to invite member {member_id} to room {room_id}: {error}");
            RoomCommandError::ServiceUnavailable
        })?;

        Ok(InviteMemberResult { room_id, member_id })
    }

    async fn list_rooms(&self, limit: usize, offset: usize) -> ListRoomsResult {
        let limit = i64::try_from(limit).unwrap_or(i64::MAX);
        let offset = i64::try_from(offset).unwrap_or(i64::MAX);

        let total = match sqlx::query("SELECT COUNT(*) AS count FROM rooms")
            .fetch_one(&self.pool)
            .await
        {
            Ok(row) => {
                let count: i64 = row.get("count");
                usize::try_from(count).unwrap_or(0)
            }
            Err(error) => {
                tracing::error!("failed to count rooms: {error}");
                0
            }
        };

        let rows = match sqlx::query(
            "SELECT r.id, r.name, r.topic, COUNT(rm.member_id) AS member_count \
             FROM rooms r \
             LEFT JOIN room_members rm ON rm.room_id = r.id \
             GROUP BY r.id, r.name, r.topic, r.created_at \
             ORDER BY r.created_at ASC \
             LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        {
            Ok(rows) => rows,
            Err(error) => {
                tracing::error!("failed to list rooms: {error}");
                Vec::new()
            }
        };

        let rooms = rows
            .into_iter()
            .map(|row| {
                let member_count: i64 = row.get("member_count");
                RoomSummary {
                    id: row.get("id"),
                    name: row.get("name"),
                    topic: row.try_get("topic").ok(),
                    member_count: usize::try_from(member_count).ok(),
                }
            })
            .collect();

        ListRoomsResult { rooms, total }
    }

    async fn delete_room(&self, id: &str) -> Result<(), RoomCommandError> {
        let result = sqlx::query("DELETE FROM rooms WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|error| {
                tracing::error!("failed to delete room {id}: {error}");
                RoomCommandError::ServiceUnavailable
            })?;

        if result.rows_affected() == 0 {
            return Err(RoomCommandError::RoomNotFound);
        }

        Ok(())
    }

    async fn export_member_data(&self, member_id: &str) -> MemberDataExport {
        let message_rows = match sqlx::query(
            "SELECT room_id, id, sender_id, content, reply_to FROM messages \
             WHERE sender_id = $1 ORDER BY created_at ASC",
        )
        .bind(member_id)
        .fetch_all(&self.pool)
        .await
        {
            Ok(rows) => rows,
            Err(error) => {
                tracing::error!("failed to export messages for member {member_id}: {error}");
                Vec::new()
            }
        };

        let messages = message_rows
            .into_iter()
            .map(|row| {
                let id: String = row.get("id");
                let text: String = row.get("content");
                MemberMessageRecord {
                    room_id: row.get("room_id"),
                    message: StoredMessage {
                        id: id.clone(),
                        sender: row.get("sender_id"),
                        text: self.decrypt_message_text(&id, text),
                        reply_to: row.try_get("reply_to").ok(),
                    },
                }
            })
            .collect();

        let room_rows = match sqlx::query("SELECT id, name, topic FROM rooms WHERE creator_id = $1")
            .bind(member_id)
            .fetch_all(&self.pool)
            .await
        {
            Ok(rows) => rows,
            Err(error) => {
                tracing::error!("failed to export rooms for member {member_id}: {error}");
                Vec::new()
            }
        };

        let rooms = room_rows
            .into_iter()
            .map(|row| MemberRoomRecord {
                id: row.get("id"),
                name: row.get("name"),
                topic: row.try_get("topic").ok(),
            })
            .collect();

        MemberDataExport { messages, rooms }
    }

    async fn delete_member_data(
        &self,
        member_id: &str,
    ) -> Result<MemberDeletionResult, RoomCommandError> {
        let mut tx = self.pool.begin().await.map_err(|error| {
            tracing::error!("failed to begin privacy deletion transaction: {error}");
            RoomCommandError::ServiceUnavailable
        })?;

        let message_result = sqlx::query(
            "UPDATE messages \
             SET content = '[Content removed per GDPR request]', sender_id = '[deleted]' \
             WHERE sender_id = $1",
        )
        .bind(member_id)
        .execute(&mut *tx)
        .await
        .map_err(|error| {
            tracing::error!("failed to anonymize messages for member {member_id}: {error}");
            RoomCommandError::ServiceUnavailable
        })?;

        sqlx::query("DELETE FROM room_members WHERE member_id = $1")
            .bind(member_id)
            .execute(&mut *tx)
            .await
            .map_err(|error| {
                tracing::error!(
                    "failed to remove room memberships for member {member_id}: {error}"
                );
                RoomCommandError::ServiceUnavailable
            })?;

        let room_rows = sqlx::query("SELECT id FROM rooms WHERE creator_id = $1")
            .bind(member_id)
            .fetch_all(&mut *tx)
            .await
            .map_err(|error| {
                tracing::error!("failed to select rooms for member {member_id}: {error}");
                RoomCommandError::ServiceUnavailable
            })?;
        let rooms_created = i64::try_from(room_rows.len()).unwrap_or(i64::MAX);

        sqlx::query("DELETE FROM rooms WHERE creator_id = $1")
            .bind(member_id)
            .execute(&mut *tx)
            .await
            .map_err(|error| {
                tracing::error!("failed to delete rooms for member {member_id}: {error}");
                RoomCommandError::ServiceUnavailable
            })?;

        tx.commit().await.map_err(|error| {
            tracing::error!("failed to commit privacy deletion transaction: {error}");
            RoomCommandError::ServiceUnavailable
        })?;

        Ok(MemberDeletionResult {
            messages: i64::try_from(message_result.rows_affected()).unwrap_or(i64::MAX),
            rooms_created,
        })
    }
}
