use crate::{
    contracts::{DesktopError, Result},
    storage::atomic_json,
};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskPolicy {
    pub background_concurrency: usize,
    pub ocr_concurrency: usize,
    pub tts_concurrency: usize,
    pub max_text_requests: u32,
    pub max_content_fix_rounds: u32,
}
impl Default for TaskPolicy {
    fn default() -> Self {
        Self {
            background_concurrency: 2,
            ocr_concurrency: 2,
            tts_concurrency: 2,
            max_text_requests: 40,
            max_content_fix_rounds: 2,
        }
    }
}
impl TaskPolicy {
    pub fn validate(&self) -> Result<()> {
        if !(1..=4).contains(&self.background_concurrency)
            || !(1..=2).contains(&self.ocr_concurrency)
            || !(1..=3).contains(&self.tts_concurrency)
            || !(1..=1000).contains(&self.max_text_requests)
            || !(1..=3).contains(&self.max_content_fix_rounds)
        {
            return Err(DesktopError::new("task_policy","后台并发须为1–4、OCR并发1–2、TTS并发1–3，文字/评测请求预算须为1–1000，内容修复轮数须为1–3"));
        }
        Ok(())
    }
    pub fn load(root: &Path) -> Result<Self> {
        let path = root.join("task-policy.json");
        let value: Self = if path.is_file() {
            serde_json::from_slice(&fs::read(path)?)?
        } else {
            Self::default()
        };
        value.validate()?;
        Ok(value)
    }
    pub fn save(&self, root: &Path) -> Result<()> {
        self.validate()?;
        atomic_json(&root.join("task-policy.json"), self)
    }
}
