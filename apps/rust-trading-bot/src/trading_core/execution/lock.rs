use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct TradingLock {
    pub symbol: String,
    pub operation: String, // "close", "open_long", "open_short"
    pub process_name: String,
    pub created_at: u64,
    pub expires_at: u64,
}

pub struct TradingLockManager {
    lock_dir: String,
}

impl Default for TradingLockManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TradingLockManager {
    pub fn new() -> Self {
        Self {
            lock_dir: "./trading_locks".to_string(),
        }
    }

    fn ensure_lock_dir(&self) -> Result<()> {
        if !Path::new(&self.lock_dir).exists() {
            fs::create_dir_all(&self.lock_dir)?;
        }
        Ok(())
    }

    fn get_lock_file(&self, symbol: &str, operation: &str) -> String {
        format!("{}/{}_{}.lock", self.lock_dir, symbol, operation)
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// 尝试获取交易锁，如果成功返回true，如果已被锁定返回false
    pub fn try_acquire_lock(
        &self,
        symbol: &str,
        operation: &str,
        process_name: &str,
        timeout_seconds: u64,
    ) -> Result<bool> {
        self.ensure_lock_dir()?;

        let lock_file = self.get_lock_file(symbol, operation);
        let now = Self::current_timestamp();

        // 检查现有锁是否过期
        if Path::new(&lock_file).exists() {
            if let Ok(content) = fs::read_to_string(&lock_file) {
                if let Ok(existing_lock) = serde_json::from_str::<TradingLock>(&content) {
                    if existing_lock.expires_at > now {
                        // 锁仍然有效
                        return Ok(false);
                    }
                    // 锁已过期，可以删除
                    fs::remove_file(&lock_file).ok();
                }
            }
        }

        // 创建新锁
        let lock = TradingLock {
            symbol: symbol.to_string(),
            operation: operation.to_string(),
            process_name: process_name.to_string(),
            created_at: now,
            expires_at: now + timeout_seconds,
        };

        let lock_data = serde_json::to_string_pretty(&lock)?;
        fs::write(&lock_file, lock_data)?;

        Ok(true)
    }

    /// 释放交易锁
    pub fn release_lock(&self, symbol: &str, operation: &str, process_name: &str) -> Result<()> {
        let lock_file = self.get_lock_file(symbol, operation);

        if Path::new(&lock_file).exists() {
            if let Ok(content) = fs::read_to_string(&lock_file) {
                if let Ok(existing_lock) = serde_json::from_str::<TradingLock>(&content) {
                    // 只有创建锁的进程才能释放
                    if existing_lock.process_name == process_name {
                        fs::remove_file(&lock_file)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// 检查特定操作是否被锁定
    pub fn is_locked(&self, symbol: &str, operation: &str) -> bool {
        let lock_file = self.get_lock_file(symbol, operation);
        let now = Self::current_timestamp();

        if Path::new(&lock_file).exists() {
            if let Ok(content) = fs::read_to_string(&lock_file) {
                if let Ok(existing_lock) = serde_json::from_str::<TradingLock>(&content) {
                    return existing_lock.expires_at > now;
                }
            }
        }

        false
    }

    /// 清理所有过期的锁
    pub fn cleanup_expired_locks(&self) -> Result<()> {
        self.ensure_lock_dir()?;
        let now = Self::current_timestamp();

        if let Ok(entries) = fs::read_dir(&self.lock_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "lock") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(lock) = serde_json::from_str::<TradingLock>(&content) {
                            if lock.expires_at <= now {
                                fs::remove_file(&path).ok();
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
