use std::fs;

use crate::{domain::Contact, prelude::AppError};

pub struct AppState {
    pub path: String,
}

impl AppState {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }

    pub fn load(&self) -> Result<Vec<Contact>, AppError> {
        let data = fs::read_to_string(&self.path)
            .map_err(|e| AppError::Parse(format!("Read error: {}", e)))?;

        let contacts: Vec<Contact> = serde_json::from_str(&data)
            .map_err(|e| AppError::Parse(format!("JSON error: {}", e)))?;

        Ok(contacts)
    }

    pub fn save(&self, contacts: &[Contact]) -> Result<(), AppError> {
        let serialized = serde_json::to_string_pretty(contacts)
            .map_err(|e| AppError::Parse(format!("Serialize error: {}", e)))?;

        fs::write(&self.path, serialized)
            .map_err(|e| AppError::Parse(format!("Write error: {}", e)))?;

        Ok(())
    }
}
