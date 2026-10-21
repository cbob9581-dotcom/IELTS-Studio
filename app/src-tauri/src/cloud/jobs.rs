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

pub fn authorize_audio(j: &mut Value) -> Result<()> {
    if j["kind"] != "generation" || j["state"] != "review_required" || !content_approved(j) {
        return Err(error("题目和独立文字校审通过后，才能单独选择合成听力；当前不会调用 TTS"));
    }
    let fingerprint=j["textFingerprint"].as_str().filter(|s|!s.is_empty()).ok_or_else(||error("请先继续此旧任务，完成新版文字校验；不会调用 TTS"))?.to_owned();
    if j["audioPending"]!=true {return Err(error("当前草稿没有待合成的听力音频"));}
    j["audioAuthorization"]=json!({"materialHash":fingerprint,"reviewHash":j["validationHash"],"authorizedAt":now_ms()});
    j["state"]=json!("pending");j["error"]=Value::Null;
    Ok(())
}
pub fn check_audio_authorization(j: &Value, fingerprint: &str) -> Result<()> {
    if !content_approved(j) || j["audioAuthorization"]["materialHash"]!=fingerprint || j["audioAuthorization"]["reviewHash"]!=j["validationHash"] {
        return Err(error("尚未明确选择合成当前已校审文本，或文本已更改；TTS 请求未发送"));
    }
    Ok(())
}
pub fn retry_text(root:&Path,j:&mut Value)->Result<()> {
 let stage=j["stage"].as_u64().unwrap_or(0) as usize;
 if j["kind"]!="generation"||!["failed","paused"].contains(&j["state"].as_str().unwrap_or(""))||stage>=super::generation::stages(&j["request"])?.len(){return Err(error("此操作只重新请求失败的题目文字部分，不用于重做录音或整份试卷"));}
 let folder=dir(root,j["id"].as_str().ok_or_else(||error("任务编号缺失"))?)?;
 let archive=format!("history/text-retry-{}",uuid::Uuid::new_v4());let target=folder.join(&archive);fs::create_dir_all(&target)?;
 atomic_json(&target.join("previous-status.json"),&summary(j))?;
 let prefix=format!("part-{stage}");for item in fs::read_dir(&folder)?{let item=item?;let name=item.file_name().to_string_lossy().to_string();if item.path().is_file()&&(name==format!("{prefix}.json")||name.starts_with(&format!("{prefix}-"))||name.starts_with(&format!("{prefix}."))){fs::rename(item.path(),target.join(name))?;}}
 j["lastTextRetryArchive"]=json!(archive);j["state"]=json!("pending");j["error"]=Value::Null;j["audioAuthorization"]=Value::Null;j["validation"]=Value::Null;j["textFingerprint"]=Value::Null;Ok(())
}
pub fn schedule_recheck(j:&mut Value)->Result<()>{if j["kind"]!="generation"||j["state"]!="review_required"||!j["paper"].is_object(){return Err(error("仅可重新校审待核对的生成卷"));}j["recheckOnly"]=json!(true);j["state"]=json!("pending");j["validation"]=Value::Null;j["validationHash"]=Value::Null;j["error"]=Value::Null;Ok(())}
pub fn retry_assessment(root:&Path,j:&mut Value)->Result<()> {
 if j["kind"]!="assessment"||!["failed","paused"].contains(&j["state"].as_str().unwrap_or("")){return Err(error("只能重新请求当前未完成的评测，已完成报告保留"));}
 let folder=dir(root,j["id"].as_str().ok_or_else(||error("任务编号缺失"))?)?;let attempt:Value=serde_json::from_slice(&fs::read(folder.join("attempt.json"))?)?;let done=j["evaluations"].as_array().map_or(0,Vec::len);
 let raw=if attempt["section"]=="speaking"{"speaking-session.response.txt".to_string()}else if attempt["section"]=="writing"{format!("writing-{done}-response.txt")}else{return Err(error("此科目不需要云端主观评测"));};
 let archive=format!("history/assessment-retry-{}",uuid::Uuid::new_v4());let target=folder.join(&archive);fs::create_dir_all(&target)?;atomic_json(&target.join("previous-status.json"),&summary(j))?;
 let response=folder.join(raw);for path in [response.clone(),response.with_extension("request.json")]{if path.exists(){fs::rename(&path,target.join(path.file_name().unwrap()))?;}}
 j["lastAssessmentRetryArchive"]=json!(archive);j["stage"]=json!(if attempt["section"]=="writing"{done}else{0});j["state"]=json!("pending");j["error"]=Value::Null;Ok(())
}
