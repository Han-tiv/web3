//! Configuration Module
//!
//! 配置和数据库管理

pub mod app_config;
pub mod database;
pub mod settings; // ✨ 新增：中心化配置

pub use app_config::AppConfig;
pub use database::Database;
