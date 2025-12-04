//! Configuration Module
//!
//! 配置和数据库管理

pub mod database;
pub mod settings;
pub mod app_config;  // ✨ 新增：中心化配置

pub use database::Database;
pub use app_config::AppConfig;
