pub mod layout;
pub mod matching;
pub mod publication;
pub mod pdf_batch;
pub mod page_image;
pub mod outline;
pub mod question_check;
pub mod transcripts;
pub mod audio_assets;
pub mod audio_jobs;
use crate::{
    contracts::{now_ms, valid_id, DesktopError, Result},
    storage::atomic_json,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};
pub fn err(s: impl ToString) -> DesktopError {
    DesktopError::new("source_library", s)
}
pub fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
pub fn dir(root: &Path, id: &str) -> Result<PathBuf> {
    if !valid_id(id) {
        return Err(err("资料项目标识无效"));
    }
    Ok(root.join("source-library").join(id))
}
pub fn load(root: &Path, id: &str) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(
        dir(root, id)?.join("project.json"),
    )?)?)
}
pub fn create(root: &Path, title: &str) -> Result<Value> {
    if title.trim().is_empty() || title.len() > 500 {
        return Err(err("请填写资料名称（不超过 500 字节）"));
    }
    let p = json!({"id":id(),"title":title.trim(),"testType":"academic","revision":0,"createdAt":now_ms(),"documents":[],"units":[],"entries":[],"publications":[]});
    atomic_json(
        &dir(root, p["id"].as_str().unwrap())?.join("project.json"),
        &p,
    )?;
    Ok(p)
}
pub fn save(root: &Path, p: &mut Value, expected: u64) -> Result<()> {
    let old = load(root, p["id"].as_str().unwrap())?;
    if old["revision"] != expected {
        return Err(err("资料已更新，请刷新后重试；当前编辑未覆盖其他修改"));
    }
    p["revision"] = json!(expected + 1);
    p["updatedAt"] = json!(now_ms());
    atomic_json(
        &dir(root, p["id"].as_str().unwrap())?.join("project.json"),
        p,
    )
}
pub fn list(root: &Path) -> Result<Vec<Value>> {
    let base = root.join("source-library");
    if !base.exists() {
        return Ok(vec![]);
    }
    let mut out = vec![];
    for e in fs::read_dir(base)? {
        let e = e?;
        if e.path().join("project.json").is_file() {
            let p: Value = serde_json::from_slice(&fs::read(e.path().join("project.json"))?)?;
            out.push(json!({"id":p["id"],"title":p["title"],"revision":p["revision"],"documents":p["documents"].as_array().map(Vec::len),"units":p["units"].as_array().map(Vec::len)}));
        }
    }
    Ok(out)
}
pub fn document_path(root: &Path, p: &Value, doc: &Value) -> Result<PathBuf> {
    let rel = doc["path"].as_str().ok_or_else(|| err("原件路径缺失"))?;
    if !crate::papers::safe_path(rel) {
        return Err(err("原件路径无效"));
    }
    let base = dir(root, p["id"].as_str().unwrap())?.canonicalize()?;
    let path = base.join(rel).canonicalize()?;
    if !path.starts_with(base) {
        return Err(err("原件路径越界"));
    }
    Ok(path)
}
pub fn add_document(root: &Path, p: &mut Value, path: &Path, roles: Value) -> Result<String> {
    let roles = roles
        .as_array()
        .filter(|a| !a.is_empty())
        .ok_or_else(|| err("请选择资料用途"))?;
    if roles.iter().any(|r| {
        !matches!(
            r.as_str(),
            Some(
                "questions"
                    | "answers"
                    | "explanations"
                    | "essays"
                    | "transcript"
                    | "audio"
                    | "mixed"
            )
        )
    }) {
        return Err(err("资料用途无效"));
    }
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ![
        "pdf", "png", "jpg", "jpeg", "webp", "txt", "md", "wav", "mp3", "ogg",
    ]
    .contains(&ext.as_str())
    {
        return Err(err("资料支持 PDF、图片、UTF-8 文本及 WAV/MP3/OGG"));
    }
    if fs::metadata(path)?.len() > 200 * 1024 * 1024 {
        return Err(err("单份资料超过 200MB，请按 Test 分卷后加入"));
    }
    if ["wav","mp3","ogg"].contains(&ext.as_str())&&fs::metadata(path)?.len()>100*1024*1024{return Err(err("听力音频单文件上限100MB，请压缩为MP3或按Part拆分后上传；原件不会修改"));}
    let mut input = fs::File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let n = input.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        digest.update(&buffer[..n]);
    }
    let hash = format!("{:x}", digest.finalize());
    if let Some(d) = p["documents"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|d| d["sha256"] == hash)
    {
        for r in roles {
            if !d["roles"].as_array().unwrap().contains(r) {
                d["roles"].as_array_mut().unwrap().push(r.clone());
            }
        }
        return Ok(d["id"].as_str().unwrap().into());
    }
    let did = id();
    let rel = format!("originals/{hash}.{ext}");
    let target = dir(root, p["id"].as_str().unwrap())?.join(&rel);
    fs::create_dir_all(target.parent().unwrap())?;
    // Copy after hashing, verify the copied bytes in case the source changed during selection.
    fs::copy(path, &target)?;
    let mut f = fs::File::open(&target)?;
    let mut h = Sha256::new();
    loop {
        let n = f.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        h.update(&buffer[..n]);
    }
    if format!("{:x}", h.finalize()) != hash {
        return Err(err("源文件在导入时发生变化，请重新选择"));
    }
    let audio = ["wav", "mp3", "ogg"].contains(&ext.as_str());
    p["documents"].as_array_mut().unwrap().push(json!({"id":did,"name":path.file_name().unwrap().to_string_lossy(),"path":rel,"sha256":hash,"extension":ext,"roles":roles,"state":if audio{"media"}else{"pending"},"scanCursor":0,"blockCount":0,"pageCount":0}));
    Ok(did)
}
pub fn blocks(root: &Path, p: &Value, doc_id: &str) -> Result<Vec<Value>> {
    if !valid_id(doc_id) {
        return Err(err("文档标识无效"));
    }
    Ok(serde_json::from_slice(&fs::read(
        dir(root, p["id"].as_str().unwrap())?
            .join("blocks")
            .join(format!("{doc_id}.json")),
    )?)?)
}
pub fn save_blocks(root: &Path, p: &Value, doc_id: &str, blocks: &[Value]) -> Result<()> {
    if !valid_id(doc_id) {
        return Err(err("文档标识无效"));
    }
    atomic_json(
        &dir(root, p["id"].as_str().unwrap())?
            .join("blocks")
            .join(format!("{doc_id}.json")),
        &blocks,
    )
}
pub fn make_unit(p: &Value, doc_id: &str, spec: &Value, blocks: &[Value]) -> Result<Value> {
    let start = spec["start"]
        .as_u64()
        .ok_or_else(|| err("请选择起始版面块"))? as usize;
    let end = spec["end"]
        .as_u64()
        .ok_or_else(|| err("请选择结束版面块"))? as usize;
    if start > end || end >= blocks.len() {
        return Err(err("题组范围超出文档"));
    }
    let test = spec["test"].as_str().unwrap_or("").trim();
    if test.is_empty() || test.len() > 100 {
        return Err(err("请指定 Test（例如 Test 1）"));
    }
    if !["reading", "listening", "writing", "speaking"]
        .contains(&spec["section"].as_str().unwrap_or(""))
    {
        return Err(err("请选择科目"));
    }
    if ![
        "questions",
        "answers",
        "explanations",
        "essays",
        "transcript",
    ]
    .contains(&spec["role"].as_str().unwrap_or(""))
    {
        return Err(err("请选择题组用途"));
    }
    let part = spec["part"].as_u64().unwrap_or(1);
    if !(1..=4).contains(&part) {
        return Err(err("Part 范围为 1–4"));
    }
    let text_len: usize = blocks[start..=end]
        .iter()
        .map(|b| b["text"].as_str().unwrap_or("").len())
        .sum();
    if text_len > 90000 {
        return Err(err(
            "此题组文本超过 90KB，请按 Passage/Task 拆开，避免输出截断",
        ));
    }
    let fingerprint=crate::papers::hash(serde_json::to_vec(&json!({"doc":doc_id,"blocks":&blocks[start..=end],"test":test,"section":spec["section"],"part":part,"role":spec["role"]}))?.as_slice());
    Ok(
        json!({"id":id(),"documentId":doc_id,"test":test,"section":spec["section"],"part":part,"role":spec["role"],"title":spec["title"].as_str().unwrap_or(test),"start":start,"end":end,"state":"pending","confirmed":false,"fingerprint":fingerprint,"projectId":p["id"]}),
    )
}

pub fn finish_step(p:&mut Value,next:Value,target:&str,failure:Option<DesktopError>)->Result<()> {
    if let Some(e)=failure {
        let key=if p["documents"].as_array().into_iter().flatten().any(|d|d["id"]==target){"documents"}else{"units"};
        let item=p[key].as_array_mut().into_iter().flatten().find(|d|d["id"]==target).ok_or_else(||err("资料条目不存在"))?;
        item["state"]=json!("failed");item["error"]=json!(e.message);
    }else{*p=next;} Ok(())
}
