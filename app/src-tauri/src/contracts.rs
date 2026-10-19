use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}
impl DesktopError {
    pub fn new(code: &str, message: impl ToString) -> Self {
        Self {
            code: code.into(),
            message: message.to_string(),
            retryable: matches!(code, "io" | "database"),
        }
    }
}
impl std::fmt::Display for DesktopError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
impl std::error::Error for DesktopError {}
impl From<std::io::Error> for DesktopError {
    fn from(e: std::io::Error) -> Self {
        Self::new("io", e)
    }
}
impl From<rusqlite::Error> for DesktopError {
    fn from(e: rusqlite::Error) -> Self {
        Self::new("database", e)
    }
}
impl From<serde_json::Error> for DesktopError {
    fn from(e: serde_json::Error) -> Self {
        Self::new("invalid_json", e)
    }
}
pub type Result<T> = std::result::Result<T, DesktopError>;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StorageRoots {
    pub assets: String,
    pub study: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttemptContract {
    pub id: String,
    pub paper_id: String,
    pub paper_version: u64,
    pub mode: String,
    pub section: String,
    pub status: String,
    pub responses: Vec<Value>,
    pub annotations: Vec<Value>,
    pub revision: u64,
    pub clock: Value,
    pub interruption_events: Vec<Value>,
}
pub fn valid_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 100
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}
pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
