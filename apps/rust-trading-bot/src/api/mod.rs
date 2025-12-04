//! API Module
//!
//! Web API 服务器和路由处理

pub mod server;
pub mod routes;

pub use server::{AppState, start_web_server};
