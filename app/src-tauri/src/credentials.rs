use crate::{
    configuration::PROVIDERS,
    contracts::{DesktopError, Result},
};
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};
static SERIAL: Mutex<()> = Mutex::new(());
static INIT: OnceLock<()> = OnceLock::new();
fn entry(provider: &str) -> Result<keyring_core::Entry> {
    if !PROVIDERS.contains(&provider) {
        return Err(DesktopError::new("credentials", "未知服务"));
    }
    if INIT.get().is_none() {
        let store = windows_native_keyring_store::Store::new()
            .map_err(|_| DesktopError::new("credentials", "Windows 凭证存储不可用"))?;
        keyring_core::set_default_store(store);
        let _ = INIT.set(());
    }
    keyring_core::Entry::new_with_modifiers(
        "local.ielts.studio",
        provider,
        &HashMap::from([("persistence", "Local")]),
    )
    .map_err(|_| DesktopError::new("credentials", "无法创建凭证条目"))
}
pub fn configured(provider: &str) -> Result<bool> {
    let _guard = SERIAL
        .lock()
        .map_err(|_| DesktopError::new("credentials", "凭证状态异常"))?;
    match entry(provider)?.get_secret() {
        Ok(secret) => Ok(!secret.is_empty()),
        Err(keyring_core::Error::NoEntry) => Ok(false),
        Err(_) => Err(DesktopError::new("credentials", "无法读取凭证状态")),
    }
}
pub fn save(provider: &str, secret: &str) -> Result<()> {
    let _guard = SERIAL
        .lock()
        .map_err(|_| DesktopError::new("credentials", "凭证状态异常"))?;
    if secret.trim().is_empty() || secret.len() > 2400 || secret.chars().any(char::is_control) {
        return Err(DesktopError::new("credentials", "密钥为空、过长或包含换行"));
    }
    entry(provider)?
        .set_password(secret)
        .map_err(|_| DesktopError::new("credentials", "写入 Windows 凭证失败；密钥未写入普通配置"))
}
pub fn clear(provider: &str) -> Result<()> {
    let _guard = SERIAL
        .lock()
        .map_err(|_| DesktopError::new("credentials", "凭证状态异常"))?;
    match entry(provider)?.delete_credential() {
        Ok(()) | Err(keyring_core::Error::NoEntry) => Ok(()),
        Err(_) => Err(DesktopError::new("credentials", "删除凭证失败")),
    }
}
// Used only by native service adapters. Never registered as an IPC command.
pub fn read_for_service(provider: &str) -> Result<String> {
    let _guard = SERIAL
        .lock()
        .map_err(|_| DesktopError::new("credentials", "凭证状态异常"))?;
    entry(provider)?
        .get_password()
        .map_err(|_| DesktopError::new("credentials", "请先在配置中保存此服务的 API 密钥"))
}
// Temporary signed upload URLs belong in OS-protected storage, never in the shareable asset library.
fn upload_entry(id: &str) -> Result<keyring_core::Entry> {
    if !crate::contracts::valid_id(id) {
        return Err(DesktopError::new("credentials", "上传任务标识无效"));
    }
    let _ = entry("mineru")?;
    keyring_core::Entry::new_with_modifiers(
        "local.ielts.studio.upload",
        id,
        &HashMap::from([("persistence", "Local")]),
    )
    .map_err(|_| DesktopError::new("credentials", "无法创建临时上传凭证"))
}
pub fn read_upload(id: &str) -> Result<Option<String>> {
    let _guard = SERIAL
        .lock()
        .map_err(|_| DesktopError::new("credentials", "凭证状态异常"))?;
    match upload_entry(id)?.get_password() {
        Ok(value) => Ok(Some(value)),
        Err(keyring_core::Error::NoEntry) => Ok(None),
        Err(_) => Err(DesktopError::new("credentials", "无法读取临时上传凭证")),
    }
}
pub fn save_upload(id: &str, value: &str) -> Result<()> {
    let _guard = SERIAL
        .lock()
        .map_err(|_| DesktopError::new("credentials", "凭证状态异常"))?;
    if value.len() > 2400 {
        return Err(DesktopError::new(
            "credentials",
            "临时上传地址过长，未写入普通文件",
        ));
    }
    upload_entry(id)?
        .set_password(value)
        .map_err(|_| DesktopError::new("credentials", "无法保存临时上传凭证"))
}
pub fn clear_upload(id: &str) -> Result<()> {
    let _guard = SERIAL
        .lock()
        .map_err(|_| DesktopError::new("credentials", "凭证状态异常"))?;
    match upload_entry(id)?.delete_credential() {
        Ok(()) | Err(keyring_core::Error::NoEntry) => Ok(()),
        Err(_) => Err(DesktopError::new("credentials", "临时上传凭证清理失败")),
    }
}
