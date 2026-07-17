use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    pub debugger_detected: Option<String>,
    pub binary_modified: Option<String>,
    pub library_modified: Option<String>,
    pub hash_mismatch: Option<String>,
    pub memory_modified: Option<String>,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            debugger_detected: Some("Terminate".into()),
            binary_modified: Some("Terminate".into()),
            library_modified: Some("Log".into()),
            hash_mismatch: Some("Log".into()),
            memory_modified: Some("Log".into()),
        }
    }
}
