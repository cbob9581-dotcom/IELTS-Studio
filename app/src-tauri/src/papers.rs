use crate::{
    contracts::{valid_id, DesktopError, Result},
    storage::Store,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};
pub fn safe_path(path: &str) -> bool {
    !path.is_empty()
        && !path.contains(['\\', ':'])
        && !path.starts_with('/')
        && !path.chars().any(char::is_control)
        && path
            .split('/')
            .all(|x| !x.is_empty() && x != "." && x != "..")
}
fn invalid(message: impl ToString) -> DesktopError {
    DesktopError::new("invalid_paper", message)
}
pub fn validate(p: &Value) -> Result<()> {
    let schema: Value = serde_json::from_str(include_str!("../../src/domain/paper.schema.json"))?;
    let validator = jsonschema::validator_for(&schema).map_err(invalid)?;
    let errors: Vec<String> = validator
        .iter_errors(p)
        .take(10)
        .map(|e| {
            let path = e.instance_path.to_string();
            let raw = e.to_string();
            let reason = if raw.contains("required property") {
                "缺少必要字段"
            } else if raw.contains("Additional properties") {
                "包含此题型不支持的字段"
            } else if raw.contains("is not of type") {
                "字段格式不正确"
            } else if raw.contains("is not one of") {
                "题型、状态或选项值不受支持"
            } else if raw.contains("minimum") || raw.contains("less than") {
                "数值低于允许范围"
            } else {
                "字段内容未通过试卷格式检查"
            };
            let parts: Vec<_> = path.split('/').collect();
            let location = if parts.get(1) == Some(&"sections") {
                let section = parts
                    .get(2)
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(0);
                let sec = &p["sections"][section];
                let label = match sec["section"].as_str().unwrap_or("") {
                    "reading" => "阅读",
                    "listening" => "听力",
                    "writing" => "写作",
                    "speaking" => "口语",
                    _ => "试卷",
                };
                if parts.get(3) == Some(&"questions") {
                    let qi = parts
                        .get(4)
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(0);
                    format!("{label} 第 {} 题", sec["questions"][qi]["number"])
                } else {
                    label.to_string()
                }
            } else {
                "试卷".to_string()
            };
            format!("{location}：{reason}（位置：{path}）")
        })
        .collect();
    if !errors.is_empty() {
        return Err(invalid(errors.join("\n")));
    }
    let mut assets = HashSet::new();
    for a in p["assets"].as_array().unwrap() {
        let id = a["id"].as_str().unwrap();
        if !assets.insert(id) {
            return Err(invalid("资源 ID 重复"));
        }
        if !safe_path(a["path"].as_str().unwrap()) {
            return Err(invalid("资源路径越界"));
        }
        if ![
            "audio/wav",
            "audio/mpeg",
            "audio/ogg",
            "image/png",
            "image/jpeg",
            "image/webp",
        ]
        .contains(&a["mediaType"].as_str().unwrap())
        {
            return Err(invalid(
                "不支持此媒体格式，请用 PNG/JPEG/WebP 或 WAV/MP3/OGG",
            ));
        }
    }
    let mut ids = HashSet::new();
    let mut sections = HashSet::new();
    for s in p["sections"].as_array().unwrap() {
        if !sections.insert(s["section"].as_str().unwrap()) {
            return Err(invalid("科目重复"));
        }
        let mut nums = HashSet::new();
        for q in s["questions"].as_array().unwrap() {
            if !ids.insert(q["id"].as_str().unwrap()) || !nums.insert(q["number"].as_u64().unwrap())
            {
                return Err(invalid("题目 ID 或题号重复"));
            }
            let kind = q["kind"].as_str().unwrap();
            if !["writing_task", "speaking_prompt"].contains(&kind)
                && (q["answer"]["accepted"]
                    .as_array()
                    .is_none_or(|x| x.is_empty())
                    || q["evidence"].as_array().unwrap().is_empty())
            {
                return Err(invalid("客观题缺答案或证据"));
            }
            if [
                "single_choice",
                "multiple_choice",
                "matching",
                "heading_match",
            ]
            .contains(&kind)
                && q["options"].as_array().is_none_or(|x| x.is_empty())
            {
                return Err(invalid("缺少选项"));
            }
            if kind == "matching" && q["slots"].as_array().is_none_or(|x| x.is_empty()) {
                return Err(invalid("缺少匹配项目"));
            }
            if kind == "diagram_label"
                && (q["imageAssetId"].as_str().is_none()
                    || q["labels"].as_array().is_none_or(|x| x.is_empty()))
            {
                return Err(invalid("缺图或标注坐标"));
            }
            for id in q["imageAssetIds"].as_array().into_iter().flatten() {
                if !assets.contains(id.as_str().unwrap_or("")) {return Err(invalid("缺少图片资源"));}
            }
            if let Some(id) = q["imageAssetId"].as_str() {
                if !assets.contains(id) {
                    return Err(invalid("缺少图片资源"));
                }
            }
        }
        for id in s["audioAssetIds"].as_array().into_iter().flatten() {
            if !assets.contains(id.as_str().unwrap()) {
                return Err(invalid("缺少音频资源"));
            }
        }
        if let Some(list)=s["audioPlaylist"].as_array(){
            let ids:Vec<_>=list.iter().map(|t|t["assetId"].clone()).collect();
            if s["audioAssetIds"]!=serde_json::json!(ids){return Err(invalid("音频播放计划与资产列表顺序不一致"));}
            let mut seen=std::collections::HashSet::new();
            for t in list {if !seen.insert(t["assetId"].as_str().unwrap_or("")){return Err(invalid("音频播放计划重复"));}crate::source_library::audio_assets::validate_binding(&serde_json::json!({"test":"published","scope":if t["part"].is_null(){"test"}else{"part"},"part":t["part"],"segments":t["segments"]}),t["durationMs"].as_u64().unwrap_or(u64::MAX))?;}
        }
    }
    Ok(())
}
pub fn version_dir(s: &Store, id: &str, version: u64) -> Result<PathBuf> {
    if !valid_id(id) || version == 0 {
        return Err(invalid("非法卷标识"));
    }
    Ok(Path::new(&s.roots.assets)
        .join("papers")
        .join(id)
        .join(version.to_string()))
}
pub fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn import(s: &Store, path: &Path) -> Result<Value> {
    if fs::metadata(path)?.len() > 20 * 1024 * 1024 {
        return Err(invalid("卷 JSON 超过 20MB"));
    }
    let p: Value = serde_json::from_slice(&fs::read(path)?)?;
    install(
        s,
        &p,
        path.parent().ok_or_else(|| invalid("卷路径无父目录"))?,
    )
}
pub fn install(s: &Store, p: &Value, source: &Path) -> Result<Value> {
    validate(p)?;
    let target = version_dir(s, p["id"].as_str().unwrap(), p["version"].as_u64().unwrap())?;
    if target.exists() {
        let old: Value = serde_json::from_slice(&fs::read(target.join("paper.json"))?)?;
        if old == *p {
            return Ok(old);
        }
        return Err(DesktopError::new(
            "immutable_version",
            "此卷版本已存在；修改后请增加 version",
        ));
    }
    let root = source.canonicalize()?;
    let stage = Path::new(&s.roots.assets).join(format!(".import-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&stage)?;
    let result = (|| {
        for a in p["assets"].as_array().unwrap() {
            let rel = a["path"].as_str().unwrap();
            let origin = root.join(rel).canonicalize()?;
            if !origin.starts_with(&root) {
                return Err(invalid("资源链接越界"));
            }
            let size = fs::metadata(&origin)?.len();
            if size > 100 * 1024 * 1024 {
                return Err(invalid("单资源超过 100MB"));
            }
            let data = fs::read(origin)?;
            if hash(&data) != a["sha256"].as_str().unwrap() {
                return Err(invalid(format!("资源哈希不匹配: {rel}")));
            }
            let dest = stage.join(rel);
            fs::create_dir_all(dest.parent().unwrap())?;
            fs::write(dest, data)?;
        }
        fs::write(stage.join("paper.json"), serde_json::to_vec_pretty(p)?)?;
        fs::create_dir_all(target.parent().unwrap())?;
        fs::rename(&stage, &target)?;
        Ok(p.clone())
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&stage);
    }
    result
}
pub fn load(s: &Store, id: &str, version: u64) -> Result<Value> {
    let p: Value =
        serde_json::from_slice(&fs::read(version_dir(s, id, version)?.join("paper.json"))?)?;
    validate(&p)?;
    Ok(p)
}
pub fn list(s: &Store) -> Result<Vec<Value>> {
    let mut out = Vec::new();
    for id in fs::read_dir(Path::new(&s.roots.assets).join("papers"))? {
        let path = id?.path();
        if !path.is_dir() {
            continue;
        }
        for v in fs::read_dir(path)? {
            let file = v?.path().join("paper.json");
            if file.exists() {
                let p: Value = serde_json::from_slice(&fs::read(file)?)?;
                validate(&p)?;
                out.push(json!({"id":p["id"],"version":p["version"],"title":p["title"],"status":p["status"],"provenance":p["provenance"],"sections":p["sections"].as_array().unwrap().iter().map(|x|x["section"].clone()).collect::<Vec<_>>() }));
            }
        }
    }
    Ok(out)
}
pub fn read_asset(s: &Store, id: &str, version: u64, asset_id: &str) -> Result<Vec<u8>> {
    let p = load(s, id, version)?;
    let a = p["assets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["id"] == asset_id)
        .ok_or_else(|| invalid("资源不存在"))?;
    let root = version_dir(s, id, version)?.canonicalize()?;
    let path = root.join(a["path"].as_str().unwrap()).canonicalize()?;
    if !path.starts_with(root) {
        return Err(invalid("资源越界"));
    }
    if fs::metadata(&path)?.len() > 100 * 1024 * 1024 {
        return Err(invalid("资源过大"));
    }
    let data = fs::read(path)?;
    if hash(&data) != a["sha256"].as_str().unwrap() {
        return Err(invalid("资源已损坏"));
    }
    Ok(data)
}
