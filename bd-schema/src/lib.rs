//! This crate contains type definitions useful both on the client and server

pub mod auth;
pub mod books;
pub mod session;

/// Type alias that corresponds to INTEGER in sqlite
pub type Integer = i64;
/// Type alias that corresponds to TEXT in sqlite
pub type Text = String;

pub const NORMAL_USER: i64 = 1;
pub const LIBRARIAN: i64 = 2;
