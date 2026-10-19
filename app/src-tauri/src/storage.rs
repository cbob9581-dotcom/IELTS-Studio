pub use crate::contracts::StorageRoots;
use crate::contracts::{DesktopError, Result};
use rusqlite::Connection;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
pub struct Store {
    pub roots: StorageRoots,
    pub db: Connection,
}
impl Store {
    pub fn reopen(roots: StorageRoots) -> Result<Self> {
        if !Path::new(&roots.assets).is_dir()
            || !Path::new(&roots.study).join("study.sqlite3").is_file()
        {
            return Err(DesktopError::new("storage_unavailable","原资产库或学习数据库不可用，请恢复原位置或在设置中重新选择；不会自动创建空记录替代旧记录"));
        }
        Self::open(roots)
    }
}
fn normalized_dir(path: &str) -> Result<PathBuf> {
    let p = Path::new(path);
    if !p.is_absolute() {
        return Err(DesktopError::new("invalid_roots", "请选择绝对目录"));
    }
    fs::create_dir_all(p)?;
    Ok(p.canonicalize()?)
}
pub fn validate_roots(roots: &StorageRoots) -> Result<()> {
    let a = normalized_dir(&roots.assets)?;
    let s = normalized_dir(&roots.study)?;
    if a.starts_with(&s) || s.starts_with(&a) {
        return Err(DesktopError::new(
            "overlapping_roots",
            "资产库与学习记录不能相同或相互嵌套",
        ));
    }
    for p in [a, s] {
        let test = p.join(format!(".write-probe-{}", uuid::Uuid::new_v4()));
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&test)?;
        f.write_all(b"ok")?;
        f.sync_all()?;
        drop(f);
        fs::remove_file(test)?;
    }
    Ok(())
}
impl Store {
    pub fn open(roots: StorageRoots) -> Result<Self> {
        validate_roots(&roots)?;
        fs::create_dir_all(Path::new(&roots.assets).join("papers"))?;
        let db = Connection::open(Path::new(&roots.study).join("study.sqlite3"))?;
        db.busy_timeout(std::time::Duration::from_secs(5))?;
        db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;")?;
        db.execute_batch(include_str!("../migrations/001_core.sql"))?;
        let version: String = db.query_row(
            "SELECT value FROM meta WHERE key='schema_version'",
            [],
            |r| r.get(0),
        )?;
        if version != "1" {
            return Err(DesktopError::new("schema_version", "学习数据库版本不兼容"));
        }
        Ok(Self { roots, db })
    }
}
pub fn atomic_json(path: &Path, value: &impl serde::Serialize) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| DesktopError::new("path", "配置路径无父目录"))?;
    fs::create_dir_all(parent)?;
    let tmp = parent.join(format!(".pending-{}.json", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut f = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&tmp)?;
        f.write_all(&serde_json::to_vec_pretty(value)?)?;
        f.sync_all()?;
        drop(f);
        fs::rename(&tmp, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(tmp);
    }
    result
}
