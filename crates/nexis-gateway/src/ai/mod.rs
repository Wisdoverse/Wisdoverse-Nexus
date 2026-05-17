//! AI bounded context.

mod application;
mod domain;

pub use application::{AiApplication, AiApplicationConfig};
pub use domain::{AiMention, AiResponse};
