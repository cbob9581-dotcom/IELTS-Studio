use super::error;
use crate::{
    contracts::{now_ms, valid_id, Result},
    storage::atomic_json,
};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};
pub fn dir(root: &Path, id: &str) -> Result<PathBuf> {
    if !valid_id(id) {
        return Err(error("任务标识无效"));
    }
    let base = root.join("cloud-jobs");
    if base.join(id).exists() {
        return Ok(base.join(id));
    }
    for kind in ["generation", "import", "assessment"] {
        let p = base.join(kind).join(id);
        if p.exists() {
            return Ok(p);
        }
    }
    Ok(base.join(id))
}
pub fn create(root: &Path, kind: &str, request: Value) -> Result<Value> {
    if !["generation", "import", "assessment"].contains(&kind) {
        return Err(error("任务类型无效"));
    }
    let v = json!({"id":uuid::Uuid::new_v4().to_string(),"kind":kind,"state":"pending","stage":0,"createdAt":now_ms(),"updatedAt":now_ms(),"request":request,"steps":[],"error":null});
    let folder = root
        .join("cloud-jobs")
        .join(kind)
        .join(v["id"].as_str().unwrap());
    fs::create_dir_all(&folder)?;
    atomic_json(&folder.join("job.json"), &v)?;
    atomic_json(&folder.join("status.json"), &summary(&v))?;
    fs::write(folder.join("说明.txt"),"此目录由 IELTS Studio 管理，一份任务对应一个目录。\njob.json：可恢复草稿和状态；part-*.json：已完成部分；media：录音及图片；source.*：原资料；original.md/cleaned.md：原文/清洗结果。\n正式试卷发布到资产目录 papers。请在应用的建卷工作区查看并核对。历史任务和原资料不会自动删除。\n")?;
    Ok(v)
}
pub fn save(root: &Path, v: &Value) -> Result<()> {
    let folder = dir(root, v["id"].as_str().ok_or_else(|| error("任务无 ID"))?)?;
    atomic_json(&folder.join("job.json"), v)?;
    atomic_json(&folder.join("status.json"), &summary(v))
}
pub fn load(root: &Path, id: &str) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(
        dir(root, id)?.join("job.json"),
    )?)?)
}
pub fn list(root: &Path) -> Result<Vec<Value>> {
    let folder = root.join("cloud-jobs");
    if !folder.exists() {
        return Ok(vec![]);
    }
    let mut result = Vec::new();
    fn read_folder(folder: &Path, result: &mut Vec<Value>, nested: bool) -> Result<()> {
        for e in fs::read_dir(folder)? {
            let path = e?.path();
            let p = path.join("job.json");
            if p.is_file() {
                let read_valid = |p: &Path| {
                    fs::read(p)
                        .ok()
                        .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
                        .filter(|v| v["id"].as_str().is_some_and(valid_id))
                };
                if let Some(v) = read_valid(&path.join("status.json")).or_else(|| read_valid(&p)) {
                    result.push(v);
                }
            } else if !nested
                && path.is_dir()
                && ["generation", "import", "assessment"]
                    .iter()
                    .any(|k| path.file_name().is_some_and(|n| n == *k))
            {
                read_folder(&path, result, true)?;
            }
        }
        Ok(())
    }
    read_folder(&folder, &mut result, false)?;
    result.sort_by_key(|v| std::cmp::Reverse(v["createdAt"].as_u64().unwrap_or(0)));
    Ok(result)
}
pub fn summary(v: &Value) -> Value {
    let mut out = serde_json::Map::new();
    for key in [
        "id",
        "kind",
        "state",
        "stage",
        "createdAt",
        "updatedAt",
        "request",
        "error",
        "operation",
        "progress",
        "elapsedMs",
        "lastStepMs",
        "cleanedChunks",
        "cleanTotal",
        "warning",
        "paperTitle",
    ] {
        if let Some(value) = v.get(key) {
            out.insert(key.into(), value.clone());
        }
    }
    if let Some(p) = v.get("paper") {
        out.insert("paperTitle".into(), p["title"].clone());
    }
    Value::Object(out)
}
pub fn operation(
    root: &Path,
    j: &mut Value,
    phase: &str,
    label: &str,
    done: usize,
    total: usize,
    unit: &str,
) -> Result<()> {
    j["operation"] = json!({"phase":phase,"label":label,"done":done,"total":total,"unit":unit,"startedAt":now_ms()});
    save(root, j)
}

// An unchanged save must not be interpreted as human correction of failed content.
pub fn review_edit(j: &mut Value, paper: Value) {
    if j.get("reviewOriginal").is_none() {
        j["reviewOriginal"] = j["paper"].clone();
    }
    j["humanEditedAt"] = if paper != j["reviewOriginal"] {
        json!(now_ms())
    } else {
        Value::Null
    };
    if paper != j["paper"] && j["kind"] == "generation" {
        j["validation"] = json!({"pass":false,"issues":[{"questionId":"全卷","reason":"草稿内容已修改，请重新校审当前版本。"}]});
    }
    j["paper"] = paper;
}

pub fn content_approved(j: &Value) -> bool {
    j["validation"]["pass"] == true
        && j["validation"]["issues"]
            .as_array()
            .is_some_and(|x| x.is_empty())
        && j["validationHash"].as_str().is_some_and(|h| {
            h == crate::papers::hash(&serde_json::to_vec(&j["paper"]).unwrap_or_default())
        })
}
