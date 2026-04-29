//! Reminder domain types and services.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Reminder category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReminderType {
    BeforeDue,
    Overdue,
    Blocked,
}

/// Reminder record for a task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reminder {
    pub task_id: Uuid,
    pub remind_at: DateTime<Utc>,
    pub remind_type: ReminderType,
}

/// Reminder scheduling contract.
#[async_trait]
pub trait ReminderService: Send + Sync {
    async fn schedule(&self, reminder: Reminder);
    async fn list_for_task(&self, task_id: Uuid) -> Vec<Reminder>;
    async fn clear_for_task(&self, task_id: Uuid) -> usize;
}

/// In-memory reminder service intended for tests.
#[derive(Debug, Default, Clone)]
pub struct InMemoryReminderService {
    reminders: Arc<RwLock<Vec<Reminder>>>,
}

impl InMemoryReminderService {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ReminderService for InMemoryReminderService {
    async fn schedule(&self, reminder: Reminder) {
        self.reminders.write().await.push(reminder);
    }

    async fn list_for_task(&self, task_id: Uuid) -> Vec<Reminder> {
        self.reminders
            .read()
            .await
            .iter()
            .filter(|reminder| reminder.task_id == task_id)
            .cloned()
            .collect()
    }

    async fn clear_for_task(&self, task_id: Uuid) -> usize {
        let mut reminders = self.reminders.write().await;
        let before = reminders.len();
        reminders.retain(|reminder| reminder.task_id != task_id);
        before.saturating_sub(reminders.len())
    }
}

#[cfg(test)]
mod tests {
    use super::{InMemoryReminderService, Reminder, ReminderService, ReminderType};
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn schedules_lists_and_clears_reminders() {
        let service = InMemoryReminderService::new();
        let task_id = Uuid::new_v4();

        service
            .schedule(Reminder {
                task_id,
                remind_at: Utc::now(),
                remind_type: ReminderType::BeforeDue,
            })
            .await;

        let reminders = service.list_for_task(task_id).await;
        assert_eq!(reminders.len(), 1);
        assert_eq!(reminders[0].remind_type, ReminderType::BeforeDue);

        let cleared = service.clear_for_task(task_id).await;
        assert_eq!(cleared, 1);
        assert!(service.list_for_task(task_id).await.is_empty());
    }
}
