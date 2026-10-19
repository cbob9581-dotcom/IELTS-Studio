use crate::{
    commands::AppState,
    configuration::{self, ProviderConfig},
    contracts::{valid_id, DesktopError, Result},
    credentials, speech,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::Value;
use std::{fs, path::PathBuf};
use tauri::State;
fn config(state: &AppState, id: &str) -> Result<ProviderConfig> {
    let c = configuration::load(&state.config.with_file_name("app-settings.json"))?;
    let p = c
        .providers
        .get(id)
        .ok_or_else(|| DesktopError::new("speech", "未知云服务"))?
        .clone();
    if !p.enabled {
        return Err(DesktopError::new(
            "speech",
            "请先在主页配置中启用口语云服务",
        ));
    }
    Ok(p)
}
fn take_path(state: &AppState, attempt_id: &str, take_id: &str, writable: bool) -> Result<PathBuf> {
    if !valid_id(attempt_id) || !valid_id(take_id) {
        return Err(DesktopError::new("speech", "录音标识无效"));
    }
    let guard = state
        .store
        .lock()
        .map_err(|_| DesktopError::new("speech", "存储不可用"))?;
    let store = guard
        .as_ref()
        .ok_or_else(|| DesktopError::new("speech", "请先设置学习目录"))?;
    let attempt = crate::attempts::load(store, attempt_id)?;
    if attempt["section"] != "speaking" || (writable && attempt["status"] == "submitted") {
        return Err(DesktopError::new("speech", "此记录不允许写入口语音频"));
    }
    let dir = PathBuf::from(&store.roots.study)
        .join("recordings")
        .join(attempt_id);
    if writable {
        fs::create_dir_all(&dir)?;
    }
    Ok(dir.join(format!("{take_id}.wav")))
}
#[tauri::command]
pub fn save_speaking_recording(
    attempt_id: String,
    take_id: String,
    wav_base64: String,
    state: State<AppState>,
) -> Result<()> {
    if wav_base64.len() > 23 * 1024 * 1024 {
        return Err(DesktopError::new("speech", "录音过长"));
    }
    let bytes = STANDARD
        .decode(wav_base64)
        .map_err(|_| DesktopError::new("speech", "录音编码无效"))?;
    speech::validate_wav(&bytes)?;
    let path = take_path(&state, &attempt_id, &take_id, true)?;
    if path.exists() {
        if fs::read(&path)? == bytes {
            return Ok(());
        }
        return Err(DesktopError::new("speech", "已保存录音不可覆盖"));
    }
    let staging = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    {
        use std::io::Write;
        let mut f = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&staging)?;
        f.write_all(&bytes)?;
        f.sync_all()?;
    }
    fs::rename(staging, path)?;
    Ok(())
}
#[tauri::command]
pub async fn speech_synthesize(
    attempt_id: String,
    turn_id: String,
    text: String,
    request_id: String,
    state: State<'_, AppState>,
) -> Result<tauri::ipc::Response> {
    let p = config(&state, "qwenTts")?;
    let audio = take_path(&state, &attempt_id, &format!("{turn_id}-examiner"), true)?;
    let metadata = audio.with_extension("json");
    if audio.exists() && metadata.exists() {
        let saved: Value = serde_json::from_slice(&fs::read(&metadata)?)?;
        if saved["text"] == text && saved["model"] == p.model && saved["voice"] == p.voice {
            return Ok(tauri::ipc::Response::new(fs::read(audio)?));
        }
    }
    let key = credentials::read_for_service("qwenTts")?;
    let bytes = speech::cancellable(&request_id, speech::synthesize(&p, &key, &text)).await?;
    if bytes.is_empty() || bytes.len() > 16 * 1024 * 1024 {
        return Err(DesktopError::new("speech", "TTS 未返回有效音频"));
    }
    // Write the cache only for a still-active attempt. Cancelled responses never reach this point.
    let path = take_path(&state, &attempt_id, &format!("{turn_id}-examiner"), true)?;
    fs::write(path, &bytes)?;
    crate::storage::atomic_json(
        &metadata,
        &serde_json::json!({"text":text,"model":p.model,"voice":p.voice}),
    )?;
    Ok(tauri::ipc::Response::new(bytes))
}
#[tauri::command]
pub async fn speech_recognize(
    attempt_id: String,
    take_id: String,
    request_id: String,
    state: State<'_, AppState>,
) -> Result<String> {
    let p = config(&state, "qwenAsr")?;
    let key = credentials::read_for_service("qwenAsr")?;
    let path = take_path(&state, &attempt_id, &take_id, false)?;
    let bytes = fs::read(path)?;
    speech::validate_wav(&bytes)?;
    speech::cancellable(&request_id, speech::recognize(&p, &key, &bytes)).await
}
#[tauri::command]
pub async fn speech_examiner(
    prompt: String,
    previous: String,
    part: u32,
    request_id: String,
    state: State<'_, AppState>,
) -> Result<String> {
    let p = config(&state, "deepseek")?;
    let key = credentials::read_for_service("deepseek")?;
    speech::cancellable(
        &request_id,
        speech::examiner(&p, &key, &prompt, &previous, part),
    )
    .await
}
#[tauri::command]
pub fn cancel_speech(request_id: String) {
    speech::cancel(&request_id);
}

#[tauri::command]
pub fn read_speaking_recording(
    attempt_id: String,
    take_id: String,
    state: State<AppState>,
) -> Result<tauri::ipc::Response> {
    let path = take_path(&state, &attempt_id, &take_id, false)?;
    let bytes = fs::read(path)?;
    speech::validate_wav(&bytes)?;
    Ok(tauri::ipc::Response::new(bytes))
}
#[tauri::command]
pub fn recover_speaking_recording(
    attempt_id: String,
    take_id: String,
    chunks: Vec<String>,
    state: State<AppState>,
) -> Result<Value> {
    if chunks.is_empty() || chunks.len() > 60 {
        return Err(DesktopError::new(
            "speech",
            "没有可恢复的录音片段或片段数过多",
        ));
    }
    let path = take_path(&state, &attempt_id, &take_id, true)?;
    let mut data = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for id in chunks {
        if !seen.insert(id.clone()) {
            return Err(DesktopError::new("speech", "录音片段重复"));
        }
        data.push(fs::read(take_path(&state, &attempt_id, &id, false)?)?);
    }
    let bytes = speech::join_waves(&data)?;
    if path.exists() {
        if fs::read(&path)? != bytes {
            return Err(DesktopError::new("speech", "恢复录音已存在且内容不同"));
        }
    } else {
        use std::io::Write;
        let temp = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
        let mut f = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temp)?;
        f.write_all(&bytes)?;
        f.sync_all()?;
        drop(f);
        fs::rename(temp, &path)?;
    }
    let rate = u32::from_le_bytes(bytes[24..28].try_into().unwrap());
    let voiced = crate::cloud::assessment::signal_seconds(&bytes).is_ok();
    Ok(
        serde_json::json!({"takeId":take_id,"voiced":voiced,"durationMs":(bytes.len()-44) as f64/(rate as f64*2.0)*1000.0}),
    )
}
