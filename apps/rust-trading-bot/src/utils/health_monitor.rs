use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessStatus {
    pub name: String,
    pub last_update: DateTime<Utc>,
    pub status: String,
    pub pid: u32,
}

pub struct HealthMonitor {
    status_dir: String,
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            status_dir: "./status".to_string(),
        }
    }

    pub fn ensure_status_dir(&self) -> Result<()> {
        if !Path::new(&self.status_dir).exists() {
            fs::create_dir_all(&self.status_dir)?;
        }
        Ok(())
    }

    pub fn update_status(&self, process_name: &str, status: &str) -> Result<()> {
        self.ensure_status_dir()?;

        let status_data = ProcessStatus {
            name: process_name.to_string(),
            last_update: Utc::now(),
            status: status.to_string(),
            pid: std::process::id(),
        };

        let status_file = format!("{}/{}.json", self.status_dir, process_name);
        let json_data = serde_json::to_string_pretty(&status_data)?;
        fs::write(status_file, json_data)?;

        Ok(())
    }

    pub fn get_status(&self, process_name: &str) -> Result<Option<ProcessStatus>> {
        let status_file = format!("{}/{}.json", self.status_dir, process_name);

        if !Path::new(&status_file).exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(status_file)?;
        let status: ProcessStatus = serde_json::from_str(&content)?;
        Ok(Some(status))
    }

    pub fn is_process_healthy(&self, process_name: &str, max_age_seconds: i64) -> bool {
        match self.get_status(process_name) {
            Ok(Some(status)) => {
                let now = Utc::now();
                let age = now.signed_duration_since(status.last_update).num_seconds();
                age < max_age_seconds && status.status == "running"
            }
            _ => false,
        }
    }
}
