//! Room and message application boundary for the gateway.

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod interfaces;

pub use application::{
    CreateRoomCommand, InviteMemberCommand, InviteMemberResult, ListRoomsResult, RoomApplication,
    RoomCommandError, RoomDetails, RoomRepository, SendMessageCommand,
};
pub use domain::{Room, StoredMessage};
pub use infrastructure::InMemoryRoomRepository;
#[cfg(feature = "persistence-sqlx")]
pub use infrastructure::SqlxRoomRepository;
pub use interfaces::{routes, RoomInterfaceState};
