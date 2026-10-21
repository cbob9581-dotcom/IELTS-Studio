//! A receipt is committed before a billable POST. Unknown results require explicit retry.
use super::error;
use crate::{contracts::Result, papers, storage::atomic_json};
use serde_json::{json, Value};
use std::{fs, future::Future, io::Write, path::Path};
#[derive(Clone)]
pub struct BudgetContext {
    pub root: std::path::PathBuf,
    pub task_id: String,
    pub limit: u32,
}
tokio::task_local! {pub static BUDGET:BudgetContext;}
fn meter(path: &Path, bytes: &[u8]) {
    let Ok(context) = BUDGET.try_with(Clone::clone) else {
        return;
    };
    let receipt = path.with_extension("request.json");
    let Ok(receipt) = fs::read(receipt) else {
        return;
    };
    let Ok(meta) = serde_json::from_slice::<Value>(&receipt) else {
        return;
    };
    let Some(attempt) = meta["attemptId"].as_str() else {
        return;
    };
    let usage = serde_json::from_slice::<Value>(bytes)
        .ok()
        .and_then(|v| v.get("usage").cloned())
        .or_else(|| {
            std::str::from_utf8(bytes)
                .ok()?
                .lines()
                .filter_map(|line| line.strip_prefix("data:"))
                .filter_map(|s| serde_json::from_str::<Value>(s.trim()).ok())
                .filter_map(|v| v.get("usage").filter(|u| u.is_object()).cloned())
                .last()
        });
    if let Some(usage) = usage {
        if let Ok(q) = crate::task_queue::Queue::open(&context.root) {
            let _ = q.record_usage(
                attempt,
                usage["prompt_tokens"]
                    .as_u64()
                    .or(usage["input_tokens"].as_u64()),
                usage["completion_tokens"]
                    .as_u64()
                    .or(usage["output_tokens"].as_u64()),
            );
        }
    }
}
pub async fn recorded<F, Fut>(path: &Path, identity: &Value, send: F) -> Result<Vec<u8>>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<Vec<u8>>>,
{
    let receipt = path.with_extension("request.json");
    let fingerprint = papers::hash(&serde_json::to_vec(identity)?);
    if receipt.is_file() {
        let previous: Value = serde_json::from_slice(&fs::read(&receipt)?)?;
        if previous["fingerprint"] != fingerprint {
            return Err(error("此步骤的模型、提示或参数已变化；已保留原响应，请使用明确的重新请求操作，避免覆盖已有计费结果"));
        }
    }
    if path.is_file() {
        let bytes = fs::read(path)?;
        meter(path, &bytes);
        return Ok(bytes);
    }
    if receipt.is_file() {
        let previous: Value = serde_json::from_slice(&fs::read(&receipt)?)?;
        if previous["state"] == "budget_blocked" {
            fs::remove_file(&receipt)?;
        } else {
            return Err(error("此请求已发送但结果尚未确认。已停止自动重发，请核对任务后选择重新请求；TTS 不会因此启动"));
        }
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut marker = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&receipt)
        .map_err(|_| error("此请求已有执行记录，请刷新任务，不能并发重发"))?;
    let attempt = uuid::Uuid::new_v4().to_string();
    marker.write_all(&serde_json::to_vec(&json!({"attemptId":attempt,"fingerprint":fingerprint,"state":"sending","at":crate::contracts::now_ms()}))?)?;
    marker.sync_all()?;
    drop(marker);
    if let Ok(context) = BUDGET.try_with(Clone::clone) {
        let reserve = crate::task_queue::Queue::open(&context.root).and_then(|mut q| {
            q.reserve_request(
                &context.task_id,
                &attempt,
                identity["request"]["model"].as_str().unwrap_or("unknown"),
                context.limit,
            )
        });
        if let Err(e) = reserve {
            atomic_json(
                &receipt,
                &json!({"attemptId":attempt,"fingerprint":fingerprint,"state":"budget_blocked","error":e.message}),
            )?;
            return Err(e);
        }
    }
    match send().await {
        Ok(bytes) => {
            let temp = path.with_extension(format!("pending-{}", uuid::Uuid::new_v4()));
            let mut f = fs::File::create(&temp)?;
            f.write_all(&bytes)?;
            f.sync_all()?;
            drop(f);
            fs::rename(&temp, path)?;
            atomic_json(
                &receipt,
                &json!({"attemptId":attempt,"fingerprint":fingerprint,"state":"received","at":crate::contracts::now_ms()}),
            )?;
            meter(path, &bytes);
            Ok(bytes)
        }
        Err(e) => {
            atomic_json(
                &receipt,
                &json!({"attemptId":attempt,"fingerprint":fingerprint,"state":"uncertain","at":crate::contracts::now_ms(),"error":e.message}),
            )?;
            Err(e)
        }
    }
}
