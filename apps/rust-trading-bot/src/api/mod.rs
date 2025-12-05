//! API Module
//!
//! Web API 服务器和路由处理

pub mod routes;
pub mod server;

pub use server::{start_web_server, AppState};
