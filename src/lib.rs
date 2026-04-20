pub mod config;
pub mod domain;
pub mod error;
pub mod infra;
pub mod auth;
pub mod session;
pub mod message;
pub mod group;
pub mod friend;
pub mod event;
pub mod api;
pub mod server;

pub use config::Settings;
pub use error::{AppError, AppResult, AuthError};
pub use server::{ChatServer, ServerBuilder};
