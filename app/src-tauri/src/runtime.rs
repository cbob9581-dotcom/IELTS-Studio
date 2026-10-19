use crate::contracts::{DesktopError, Result};
use serde_json::Value;
use std::path::Path;
pub fn validate_glb(bytes: &[u8]) -> Result<Value> {
    let err = || DesktopError::new("invalid_glb", "GLB 文件不完整、版本不兼容或引用外部资源");
    if bytes.len() < 24
        || bytes.len() > 256 * 1024 * 1024
        || &bytes[..4] != b"glTF"
        || u32::from_le_bytes(bytes[4..8].try_into().unwrap()) != 2
        || u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize != bytes.len()
        || &bytes[16..20] != b"JSON"
    {
        return Err(err());
    }
    let len = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    if len > bytes.len() - 20 {
        return Err(err());
    }
    let doc: Value = serde_json::from_slice(&bytes[20..20 + len])?;
    for field in ["buffers", "images"] {
        for resource in doc[field].as_array().into_iter().flatten() {
            if let Some(uri) = resource["uri"].as_str() {
                if !uri.starts_with("data:") {
                    return Err(err());
                }
            }
        }
    }
    Ok(doc)
}
pub fn read(path: &Path) -> Result<Vec<u8>> {
    if path
        .extension()
        .and_then(|e| e.to_str())
        .is_none_or(|e| !e.eq_ignore_ascii_case("glb"))
    {
        return Err(DesktopError::new("invalid_glb", "请选择独立 GLB 文件"));
    }
    if std::fs::metadata(path)?.len() > 256 * 1024 * 1024 {
        return Err(DesktopError::new("invalid_glb", "GLB 超过 256MB"));
    }
    let bytes = std::fs::read(path)?;
    validate_glb(&bytes)?;
    Ok(bytes)
}
