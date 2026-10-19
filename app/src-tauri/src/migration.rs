use crate::{
    contracts::{DesktopError, Result, StorageRoots},
    papers,
    storage::{validate_roots, Store},
};
use std::{fs, path::Path};
fn nonempty(path: &Path) -> Result<bool> {
    Ok(path.exists() && fs::read_dir(path)?.next().is_some())
}
fn copy_tree(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target)?;
    for e in fs::read_dir(source)? {
        let e = e?;
        let meta = e.file_type()?;
        if meta.is_symlink() {
            return Err(DesktopError::new("migration", "迁移不支持符号链接"));
        }
        let dest = target.join(e.file_name());
        if meta.is_dir() {
            copy_tree(&e.path(), &dest)?;
        } else {
            fs::copy(e.path(), &dest)?;
            if papers::hash(&fs::read(e.path())?) != papers::hash(&fs::read(dest)?) {
                return Err(DesktopError::new("migration", "文件校验失败"));
            }
        }
    }
    Ok(())
}
pub fn copy_verified(s: &Store, target: StorageRoots) -> Result<Store> {
    validate_roots(&target)?;
    let new_a = Path::new(&target.assets).canonicalize()?;
    let new_s = Path::new(&target.study).canonicalize()?;
    let old_a = Path::new(&s.roots.assets).canonicalize()?;
    let old_s = Path::new(&s.roots.study).canonicalize()?;
    if new_a == old_a && new_s == old_s {
        return Err(DesktopError::new("migration_conflict", "保存位置没有改变"));
    }
    for (new, previous) in [(&new_a, &old_a), (&new_s, &old_s)] {
        if new == previous {
            continue;
        }
        for old in [&old_a, &old_s] {
            if new.starts_with(old) || old.starts_with(new) {
                return Err(DesktopError::new(
                    "migration_conflict",
                    "新旧目录不能重叠或互相嵌套",
                ));
            }
        }
        if nonempty(new)? {
            return Err(DesktopError::new(
                "migration_conflict",
                "迁移目标必须是空目录",
            ));
        }
    }
    if new_a != old_a {
        copy_tree(&old_a, &new_a)?;
    }
    if new_s != old_s {
        for e in fs::read_dir(&old_s)? {
            let e = e?;
            let name = e.file_name();
            let name_str = name.to_string_lossy();
            if ["study.sqlite3", "study.sqlite3-wal", "study.sqlite3-shm"]
                .contains(&name_str.as_ref())
            {
                continue;
            }
            if e.file_type()?.is_symlink() {
                return Err(DesktopError::new("migration", "学习目录有不支持的链接"));
            }
            if e.file_type()?.is_dir() {
                copy_tree(&e.path(), &new_s.join(&name))?;
            } else {
                let bytes = fs::read(e.path())?;
                fs::write(new_s.join(&name), &bytes)?;
                if papers::hash(&fs::read(new_s.join(&name))?) != papers::hash(&bytes) {
                    return Err(DesktopError::new("migration", "学习文件校验失败"));
                }
            }
        }
        s.db.backup(
            rusqlite::DatabaseName::Main,
            new_s.join("study.sqlite3"),
            None,
        )?;
    }
    let migrated = Store::open(target)?;
    let integrity: String = migrated
        .db
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
    if integrity != "ok" {
        return Err(DesktopError::new("migration", "数据库完整性校验失败"));
    }
    Ok(migrated)
}
