//! Shared examiner speech with a durable pre-POST marker and explicit unknown-result retry.
use crate::{
    contracts::{now_ms, DesktopError, Result},
    storage::atomic_json,
};
use serde_json::json;
use std::{
    collections::HashMap,
    fs,
    future::Future,
    io::Write,
    path::Path,
    sync::{Arc, Mutex, OnceLock, Weak},
};
type Locks = Mutex<HashMap<String, Weak<tokio::sync::Mutex<()>>>>;
static LOCKS: OnceLock<Locks> = OnceLock::new();
pub async fn obtain<F, Fut>(
    root: &Path,
    key: &str,
    allow_retry: bool,
    generate: F,
) -> Result<Vec<u8>>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<Vec<u8>>>,
{
    if key.len() != 64 || !key.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err(DesktopError::new("speech_cache", "语音缓存标识无效"));
    }
    fs::create_dir_all(root)?;
    let path = root.join(format!("{key}.audio"));
    let marker = root.join(format!("{key}.attempt.json"));
    let mutex = {
        let mut locks = LOCKS
            .get_or_init(Default::default)
            .lock()
            .map_err(|_| DesktopError::new("speech_cache", "语音缓存锁异常"))?;
        locks.retain(|_, w| w.strong_count() > 0);
        let id = path.to_string_lossy().to_string();
        if let Some(lock) = locks.get(&id).and_then(Weak::upgrade) {
            lock
        } else {
            let lock = Arc::new(tokio::sync::Mutex::new(()));
            locks.insert(id, Arc::downgrade(&lock));
            lock
        }
    };
    let _guard = mutex.lock().await;
    if path.is_file() {
        let bytes = fs::read(&path)?;
        crate::cloud::generation_workflow::audio_type(&bytes)?;
        return Ok(bytes);
    }
    if marker.exists() {
        if !allow_retry {
            return Err(DesktopError::new(
                "speech_uncertain",
                "上次考官语音请求尚未确认，已停止重复合成。若选择重新请求，可能再次计费。",
            ));
        }
        fs::rename(
            &marker,
            marker.with_extension(format!("previous-{}.json", uuid::Uuid::new_v4())),
        )?;
    }
    let mut intent = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&marker)
        .map_err(|_| {
            DesktopError::new(
                "speech_uncertain",
                "同一语音请求已开始，已停止重复合成，请稍后重试",
            )
        })?;
    intent.write_all(&serde_json::to_vec(
        &json!({"state":"sending","at":now_ms()}),
    )?)?;
    intent.sync_all()?;
    drop(intent);
    let bytes = generate().await?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err(DesktopError::new(
            "speech_cache",
            "考官音频超过缓存大小限制",
        ));
    }
    crate::cloud::generation_workflow::audio_type(&bytes)?;
    let pending = path.with_extension(format!("pending-{}", uuid::Uuid::new_v4()));
    let mut f = fs::File::create(&pending)?;
    f.write_all(&bytes)?;
    f.sync_all()?;
    drop(f);
    fs::rename(&pending, &path)?;
    atomic_json(&marker, &json!({"state":"received","at":now_ms()}))?;
    Ok(bytes)
}
