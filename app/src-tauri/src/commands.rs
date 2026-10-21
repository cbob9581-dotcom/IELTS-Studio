use crate::{
    attempts,
    contracts::{DesktopError, Result, StorageRoots},
    migration, papers,
    storage::{atomic_json, Store},
};
use serde_json::Value;
use std::{fs, path::PathBuf, sync::Mutex};
use tauri::{Manager, State};
pub struct AppState {
    pub store: Mutex<Option<Store>>,
    pub config: PathBuf,
    pub startup_error: Mutex<Option<String>>,
}
pub(crate) fn locked<T>(state: &AppState, f: impl FnOnce(&Store) -> Result<T>) -> Result<T> {
    let guard = state
        .store
        .lock()
        .map_err(|_| DesktopError::new("state", "存储状态异常"))?;
    let s = guard
        .as_ref()
        .ok_or_else(|| DesktopError::new("not_configured", "请先选择资产库与学习记录目录"))?;
    f(s)
}
pub fn initialize(app: &tauri::App) -> Result<()> {
    let config = app
        .path()
        .app_config_dir()
        .map_err(|e| DesktopError::new("config", e))?
        .join("storage-roots.json");
    #[cfg(feature="verification")]
    let config=std::env::var_os("IELTS_STUDIO_TEST_CONFIG_DIR").map(|p|PathBuf::from(p).join("storage-roots.json")).unwrap_or(config);
    let result: Result<Option<Store>> = (|| {
        if config.exists() {
            let roots: StorageRoots = serde_json::from_slice(&fs::read(&config)?)?;
            Ok(Some(Store::reopen(roots)?))
        } else {
            Ok(None)
        }
    })();
    let (store, startup_error) = match result {
        Ok(s) => (s, None),
        Err(e) => (None, Some(e.to_string())),
    };
    app.manage(AppState {
        store: Mutex::new(store),
        config,
        startup_error: Mutex::new(startup_error),
    });
    Ok(())
}
#[tauri::command]
pub fn get_roots(state: State<AppState>) -> Result<Option<StorageRoots>> {
    let guard = state
        .store
        .lock()
        .map_err(|_| DesktopError::new("state", "存储状态异常"))?;
    if guard.is_none() {
        if let Some(message) = state
            .startup_error
            .lock()
            .map_err(|_| DesktopError::new("state", "状态异常"))?
            .as_ref()
        {
            return Err(DesktopError::new("storage_unavailable", message));
        }
    }
    Ok(guard.as_ref().map(|s| s.roots.clone()))
}
#[tauri::command]
pub fn set_roots(roots: StorageRoots, state: State<AppState>) -> Result<StorageRoots> {
    let mut guard = state
        .store
        .lock()
        .map_err(|_| DesktopError::new("state", "存储状态异常"))?;
    if let Some(s) = guard.as_ref() {
        if s.roots == roots {
            return Ok(roots);
        }
        return Err(DesktopError::new(
            "migration_required",
            "已有记录，请使用迁移功能更换目录",
        ));
    }
    let s = Store::open(roots.clone())?;
    seed(&s)?;
    atomic_json(&state.config, &roots)?;
    *guard = Some(s);
    Ok(roots)
}
fn seed(s: &Store) -> Result<()> {
    let source =
        std::path::Path::new(&s.roots.assets).join(format!(".seed-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(source.join("media"))?;
    let result = (|| {
        fs::write(
            source.join("media/three-test-tones.wav"),
            include_bytes!("../../test/fixtures/media/three-test-tones.wav"),
        )?;
        fs::write(
            source.join("media/library-visitors.png"),
            include_bytes!("../../test/fixtures/media/library-visitors.png"),
        )?;
        for raw in [
            include_str!("../../test/fixtures/dev-reading.json"),
            include_str!("../../test/fixtures/dev-writing.json"),
            include_str!("../../test/fixtures/dev-speaking.json"),
            include_str!("../../test/fixtures/dev-listening.json"),
        ] {
            let p: Value = serde_json::from_str(raw)?;
            papers::install(s, &p, &source)?;
        }
        Ok(())
    })();
    let _ = fs::remove_dir_all(source);
    result
}
#[tauri::command]
pub fn import_paper(path: String, state: State<AppState>) -> Result<Value> {
    locked(&state, |s| papers::import(s, std::path::Path::new(&path)))
}
#[tauri::command]
pub fn list_papers(state: State<AppState>) -> Result<Vec<Value>> {
    locked(&state, papers::list)
}
#[tauri::command]
pub fn load_paper(id: String, version: u64, state: State<AppState>) -> Result<Value> {
    locked(&state, |s| papers::load(s, &id, version))
}
#[tauri::command]
pub fn read_asset(
    id: String,
    version: u64,
    asset_id: String,
    state: State<AppState>,
) -> Result<Vec<u8>> {
    locked(&state, |s| papers::read_asset(s, &id, version, &asset_id))
}
#[tauri::command]
pub fn read_asset_binary(id:String,version:u64,asset_id:String,state:State<AppState>)->Result<tauri::ipc::Response>{
    locked(&state,|s|papers::read_asset(s,&id,version,&asset_id)).map(tauri::ipc::Response::new)
}
#[tauri::command]
pub fn save_attempt(
    snapshot: Value,
    expected_revision: u64,
    state: State<AppState>,
) -> Result<Value> {
    locked(&state, |s| attempts::save(s, snapshot, expected_revision))
}
#[tauri::command]
pub fn load_attempt(id: String, state: State<AppState>) -> Result<Value> {
    locked(&state, |s| attempts::load(s, &id))
}
#[tauri::command]
pub fn list_attempts(state: State<AppState>) -> Result<Vec<Value>> {
    locked(&state, attempts::list)
}
#[tauri::command]
pub fn submit_attempt(id: String, expected_revision: u64, state: State<AppState>) -> Result<Value> {
    locked(&state, |s| attempts::submit(s, &id, expected_revision))
}
#[tauri::command]
pub fn migrate_roots(
    target: StorageRoots,
    state: State<'_, AppState>,
) -> Result<StorageRoots> {
    let _cloud_idle=crate::cloud_commands::idle_lock()?;
    let mut guard = state
        .store
        .lock()
        .map_err(|_| DesktopError::new("state", "存储状态异常"))?;
    let old = guard
        .as_ref()
        .ok_or_else(|| DesktopError::new("not_configured", "尚未设置目录"))?;
    let next = migration::copy_verified(old, target.clone())?;
    atomic_json(&state.config, &target)?;
    *guard = Some(next);
    Ok(target)
}

#[tauri::command]
pub fn get_config(state: State<AppState>) -> Result<crate::configuration::AppConfig> {
    crate::configuration::load(&state.config.with_file_name("app-settings.json"))
}
#[tauri::command]
pub fn save_config(config: crate::configuration::AppConfig, state: State<AppState>) -> Result<()> {
    crate::configuration::save(&state.config.with_file_name("app-settings.json"), &config)
}
#[tauri::command]
pub fn credential_status() -> Result<std::collections::BTreeMap<String, bool>> {
    crate::configuration::PROVIDERS
        .into_iter()
        .map(|id| Ok((id.into(), crate::credentials::configured(id)?)))
        .collect()
}
#[tauri::command]
pub fn save_credential(provider: String, secret: String) -> Result<()> {
    crate::credentials::save(&provider, &secret)
}
#[tauri::command]
pub fn clear_credential(provider: String) -> Result<()> {
    crate::credentials::clear(&provider)
}
#[tauri::command]
pub fn read_runtime_model(
    _app: tauri::AppHandle,
    state: State<AppState>,
) -> Result<tauri::ipc::Response> {
    let config = crate::configuration::load(&state.config.with_file_name("app-settings.json"))?;
    let path = if !config.blender.runtime_asset_path.is_empty() {
        PathBuf::from(config.blender.runtime_asset_path)
    } else {
        #[cfg(debug_assertions)]
        let resource = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../rooms/speaking/runtime/speaking-room.glb");
        #[cfg(not(debug_assertions))]
        let resource = _app
            .path()
            .resource_dir()
            .map_err(|e| DesktopError::new("runtime", e))?
            .join("runtime/speaking/speaking-room.glb");
        resource
    };
    Ok(tauri::ipc::Response::new(crate::runtime::read(&path)?))
}
