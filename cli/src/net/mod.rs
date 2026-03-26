//! Network module for P2P synchronization.
//!
//! This module provides network connection implementations for the CLI,
//! specifically the TCP-based connection used for P2P synchronization.

pub mod conn;

pub use conn::TcpConn;
