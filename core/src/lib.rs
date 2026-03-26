mod db;

// Public API modules
pub mod models;
pub mod service;
pub mod sync;

pub mod dto;
pub mod error;
pub mod search;
pub mod views;
/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
