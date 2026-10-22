mod evaluate;
mod evaluate_session;
mod generate;
mod import;
use crate::{
    attempts,
    cloud::{self, assessment, error, generation, ingestion, jobs},
    commands::{locked, AppState},
    configuration::{self, ProviderConfig},
    contracts::{now_ms, valid_id, Result},
    credentials, papers, speech,
    storage::atomic_json,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use generate::audio_type;
use serde_json::{json, Value};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};
use tauri::State;
static ACTIVE: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
pub(crate) fn idle_lock() -> Result<std::sync::MutexGuard<'static, HashSet<String>>> {
    let guard = ACTIVE
        .get_or_init(Default::default)
        .lock()
        .map_err(|_| error("任务锁异常"))?;
    if !guard.is_empty() {
        return Err(error("云任务正在执行，请停止任务后再迁移目录"));
    }
    Ok(guard)
}
pub(crate) struct Lease(String);
impl Lease {
    pub(crate) fn acquire(id: &str) -> Result<Self> {
        let mut set = ACTIVE
            .get_or_init(Default::default)
            .lock()
            .map_err(|_| error("任务锁异常"))?;
        if !set.insert(id.into()) {
            return Err(error("此任务正在执行，请勿重复点击"));
        }
        Ok(Self(id.into()))
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        if let Ok(mut s) = ACTIVE.get_or_init(Default::default).lock() {
            s.remove(&self.0);
        }
    }
}
pub(crate) fn config(state: &AppState, id: &str) -> Result<ProviderConfig> {
    let c = configuration::load(&state.config.with_file_name("app-settings.json"))?;
    let p = c
        .providers
        .get(id)
        .ok_or_else(|| error("未知服务"))?
        .clone();
    if !p.enabled {
        return Err(error(format!("请在本地设置中填写并启用 {id}")));
    }
    Ok(p)
}
fn roots(state: &AppState) -> Result<(PathBuf, PathBuf)> {
    locked(state, |s| {
        Ok((
            PathBuf::from(&s.roots.assets),
            PathBuf::from(&s.roots.study),
        ))
    })
}
fn stages(req:&Value)->Result<Vec<(String,u64)>> {generation::stages(req)}
fn job_root(state: &AppState, id: &str) -> Result<PathBuf> {
    let (a, s) = roots(state)?;
    if jobs::dir(&a, id)?.join("job.json").exists() {
        Ok(a)
    } else if jobs::dir(&s, id)?.join("job.json").exists() {
        Ok(s)
    } else {
        Err(error("任务不存在，原目录可能尚未恢复"))
    }
}
#[tauri::command]
pub fn cloud_create(
    kind: String,
    request: Value,
    path: Option<String>,
    state: State<AppState>,
) -> Result<Value> {
    let (assets, study) = roots(&state)?;
    let mut request = request;
    if kind == "generation" {
        if request["blueprintSeed"].is_null(){request["blueprintSeed"]=json!(uuid::Uuid::new_v4().as_bytes()[0] as u64);}
        let plan = stages(&request)?;
        let p = config(&state, "deepseek")?;
        credentials::read_for_service("deepseek")?;
        if plan.iter().any(|(s, _)| s == "listening" || s == "reading") && p.max_tokens < 8192 {
            return Err(error("生成阅读或听力所需输出较长，请先在设置中将 DeepSeek 输出上限设为 16384（当前小于 8192），再建卷。"));
        }

    }
    if kind == "assessment" {
        let id = request["attemptId"]
            .as_str()
            .ok_or_else(|| error("缺少作答记录"))?;
        let a = locked(&state, |s| attempts::load(s, id))?;
        if a["status"] != "submitted"
            || !["writing", "speaking"].contains(&a["section"].as_str().unwrap_or(""))
        {
            return Err(error("仅评测已提交的写作/口语记录"));
        }
        request = json!({"attemptId":id,"section":a["section"]});
        if let Some(old) = jobs::list(&study)?
            .into_iter()
            .find(|j| j["kind"] == "assessment" && j["request"] == request)
        {
            return jobs::load(&study, old["id"].as_str().unwrap());
        }
    }
    let source = if kind == "import" {
        let p = PathBuf::from(path.ok_or_else(|| error("请选择本地资料"))?);
        let ext = p
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if ![
            "pdf", "doc", "docx", "ppt", "pptx", "png", "jpg", "jpeg", "webp", "md", "txt",
        ]
        .contains(&ext.as_str())
            || !p.is_file()
            || fs::metadata(&p)?.len() > (if request["sourceOnly"]==true {200} else {100}) * 1024 * 1024
        {
            return Err(error(
                "资料格式不支持或超过文件大小上限（资料库 200MB，单卷导入 100MB）",
            ));
        }
        let data = fs::read(&p)?;
        if request["sourceOnly"]!=true {config(&state, "deepseek")?; credentials::read_for_service("deepseek")?;}
        if !["md", "txt"].contains(&ext.as_str()) {
            config(&state, "mineru")?;
            credentials::read_for_service("mineru")?;
        }
        let pages = request["pageRanges"].as_str().unwrap_or("");
        if pages.len() > 100
            || !pages
                .chars()
                .all(|c| c.is_ascii_digit() || [',', '-', ' '].contains(&c))
        {
            return Err(error("页码范围请填写如 1-12,15；留空表示全部页面"));
        }
        request = json!({"sourceOnly":request["sourceOnly"]==true,"language":if request["language"]=="ch"{"ch"}else{"en"},"section":request["section"],"pageRanges":pages,"repairOcr":request["repairOcr"]!=false,"forceOcr":request["forceOcr"]==true,"sourceName":p.file_name().unwrap().to_string_lossy(),"extension":ext,"sha256":papers::hash(&data)});
        if let Some(mut old) = jobs::list(&assets)?.into_iter().find(|j| {
            j["kind"] == "import"
                && ["sha256", "section", "pageRanges", "repairOcr", "forceOcr", "sourceOnly"]
                    .iter()
                    .all(|k| j["request"][*k] == request[*k])
                && j["request"]["language"].as_str().unwrap_or("en")==request["language"].as_str().unwrap_or("en")
        }) {
            old = jobs::load(&assets, old["id"].as_str().unwrap())?;
            old["reused"] = json!(true);
            return Ok(old);
        }
        Some(data)
    } else {
        None
    };
    let job_root = if kind == "assessment" {
        &study
    } else {
        &assets
    };
    let v = jobs::create(job_root, &kind, request)?;
    let dir = jobs::dir(job_root, v["id"].as_str().unwrap())?;
    if let Some(bytes) = source {
        fs::write(
            dir.join(format!(
                "source.{}",
                v["request"]["extension"].as_str().unwrap()
            )),
            bytes,
        )?;
    }
    if kind == "assessment" {
        let a = locked(&state, |s| {
            attempts::load(s, v["request"]["attemptId"].as_str().unwrap())
        })?;
        atomic_json(&dir.join("attempt.json"), &a)?;
        fs::create_dir_all(study.join("evaluations"))?;
    }
    Ok(v)
}
#[tauri::command]
pub fn cloud_list(state: State<AppState>) -> Result<Vec<Value>> {
    let (a, s) = roots(&state)?;
    let mut jobs = jobs::list(&a)?;
    jobs.extend(crate::cloud::jobs::list(&s)?);
    jobs.sort_by_key(|j| std::cmp::Reverse(j["createdAt"].as_u64().unwrap_or(0)));
    Ok(jobs.iter().map(jobs::summary).collect())
}
#[tauri::command]
pub fn cloud_read(id: String, state: State<AppState>) -> Result<Value> {
    let a = job_root(&state, &id)?;
    jobs::load(&a, &id)
}
#[tauri::command]
pub fn cloud_cancel(id: String, state: State<AppState>) -> Result<()> {
    let a = job_root(&state, &id)?;
    let mut j = jobs::load(&a, &id)?;
    speech::cancel(&id);
    j["state"] = json!("paused");
    j["error"] = json!("已停止后续步骤。已提交的云任务可能继续处理或计费；继续时复用保存的批次。");
    jobs::save(&a, &j)
}
#[tauri::command]
pub async fn cloud_step(id: String, state: State<'_, AppState>) -> Result<Value> {
    let _lease = Lease::acquire(&id)?;
    let (assets, study) = roots(&state)?;
    let a = job_root(&state, &id)?;
    let mut j = jobs::load(&a, &id)?;
    if ["complete", "review_required", "source_required"].contains(&j["state"].as_str().unwrap_or("")) {
        if cloud::publication::ready(&j){locked(&state,|store|cloud::publication::finish(store,&a,&mut j))?;}
        return Ok(j);
    }
    j["state"] = json!("running");
    let started = std::time::Instant::now();
    j["error"] = Value::Null;
    j["updatedAt"] = json!(now_ms());
    jobs::save(&a, &j)?;
    let result = speech::cancellable(&id, async {
        match j["kind"].as_str().unwrap_or("") {
            "generation" => generate::step(&state, &a, &mut j).await,
            "import" => import::step(&state, &a, &mut j).await,
            "assessment" => evaluate::step(&state, &a, &assets, &study, &mut j).await,
            _ => Err(error("未知任务类型")),
        }
    })
    .await;
    if let Err(e) = result {
        j["state"] = json!(if e.code == "cancelled" {
            "paused"
        } else {
            "failed"
        });
        j["errorCode"] = json!(e.code);
        j["error"] = json!(e.message);
    } else if j["state"] == "running" {
        j["state"] = json!("pending");
    }
    if cloud::publication::ready(&j){
        if let Err(e)=locked(&state,|store|cloud::publication::finish(store,&a,&mut j)){
            j["state"]=json!("failed");j["error"]=json!(e.message);j["errorCode"]=json!(e.code);
        }
    }
    j["updatedAt"] = json!(now_ms());
    let elapsed = started.elapsed().as_millis() as u64;
    j["lastStepMs"] = json!(elapsed);
    j["elapsedMs"] = json!(j["elapsedMs"].as_u64().unwrap_or(0) + elapsed);
    jobs::save(&a, &j)?;
    Ok(j)
}
fn add_asset(
    j: &mut Value,
    dir: &Path,
    id: &str,
    relative: &str,
    media_type: &str,
    bytes: &[u8],
) -> Result<()> {
    let p = dir.join(relative);
    fs::create_dir_all(p.parent().unwrap())?;
    fs::write(p, bytes)?;
    if !j["assets"].is_array() {
        j["assets"] = json!([]);
    }
    let list = j["assets"].as_array_mut().unwrap();
    list.retain(|a| a["id"] != id);
    list.push(json!({"id":id,"path":relative,"sha256":papers::hash(bytes),"mediaType":media_type}));
    Ok(())
}
#[tauri::command]
pub fn cloud_review(id: String, paper: Value, state: State<AppState>) -> Result<Value> {
    let (a, _) = roots(&state)?;
    let _lease = Lease::acquire(&id)?;
    let mut j = jobs::load(&a, &id)?;
    if j["state"] != "review_required" {
        return Err(error("仅可编辑待核对的草稿"));
    }
    if !paper.is_object() || serde_json::to_vec(&paper)?.len() > 10 * 1024 * 1024 {
        return Err(error("试卷草稿无效或过大"));
    }
    if paper["id"] != j["paper"]["id"]
        || paper["version"] != j["paper"]["version"]
        || paper["provenance"] != j["paper"]["provenance"]
        || paper["assets"]
            != j["assets"]
                .as_array()
                .cloned()
                .map(Value::Array)
                .unwrap_or(json!([]))
    {
        return Err(error(
            "不可更改卷标识、版本、来源和已登记资源；请通过添加媒体按钮登记文件",
        ));
    }
    jobs::review_edit(&mut j, paper);
    jobs::save(&a, &j)?;
    Ok(j)
}
#[tauri::command]
pub fn cloud_publish(id: String, state: State<AppState>) -> Result<Value> {
    let (a, _) = roots(&state)?;
    let _lease = Lease::acquire(&id)?;
    let mut j = jobs::load(&a, &id)?;
    if j["state"] == "complete" {
        return Ok(j);
    }
    if j["state"] != "review_required" {
        return Err(error("任务未进入核对阶段"));
    }
    if j["kind"] == "generation" && !jobs::content_approved(&j) {
        return Err(error(format!("文字校审未通过，未发布且不会合成听力。具体问题：{}。可选择“AI 修复并复查（仅文字）”。",j["validation"]["issues"].as_array().into_iter().flatten().filter_map(|i|i["reason"].as_str()).collect::<Vec<_>>().join("；"))));
    }
    let mut p = j["paper"].clone();
    p["status"] = json!("ready");
    papers::validate(&p)?;
    if j["kind"] == "import" {
        ingestion::verify_answer_sources(
            &p,
            &fs::read_to_string(jobs::dir(&a, &id)?.join("original.md"))?,
        )?;
    }
    for s in p["sections"].as_array().unwrap() {
        if s["section"] == "listening" && s["audioAssetIds"].as_array().is_none_or(|a| a.is_empty())
        {
            return Err(error("听力缺少录音，请补充媒体并绑定 audioAssetIds 后发布"));
        }
    }
    let installed = locked(&state, |s| papers::install(s, &p, &jobs::dir(&a, &id)?))?;
    j["paper"] = installed;
    j["state"] = json!("complete");
    j["publishedAt"] = json!(now_ms());
    jobs::save(&a, &j)?;
    Ok(j)
}
#[tauri::command]
pub fn cloud_add_media(id: String, path: String, state: State<AppState>) -> Result<Value> {
    let (a, _) = roots(&state)?;
    let _lease = Lease::acquire(&id)?;
    let mut j = jobs::load(&a, &id)?;
    if j["state"] != "review_required" {
        return Err(error("请在待核对阶段添加资源"));
    }
    let source = Path::new(&path);
    if fs::metadata(source)?.len() > 100 * 1024 * 1024 {
        return Err(error("媒体超过100MB"));
    }
    let ext = source
        .extension()
        .and_then(|x| x.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let mime = match ext.as_str() {
        "wav" => "audio/wav",
        "mp3" => "audio/mpeg",
        "ogg" => "audio/ogg",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        _ => return Err(error("仅支持 WAV/MP3/OGG/PNG/JPEG/WebP")),
    };
    let data = fs::read(source)?;
    if mime.starts_with("audio/"){cloud::publication::validate_audio_upload(&data,&ext)?;}
    let asset_id = format!("attached-{}", uuid::Uuid::new_v4());
    let dir = jobs::dir(&a, &id)?;
    add_asset(
        &mut j,
        &dir,
        &asset_id,
        &format!("media/{asset_id}.{ext}"),
        mime,
        &data,
    )?;
    if mime.starts_with("audio/")&&j["kind"]=="generation"&&jobs::content_approved(&j){
        cloud::publication::attach_audio(&mut j,&asset_id)?;
        locked(&state,|store|cloud::publication::finish(store,&a,&mut j))?;
    }else{j["paper"]["assets"] = j["assets"].clone();}
    jobs::save(&a, &j)?;
    Ok(j)
}
#[tauri::command]
pub fn cloud_asset(
    id: String,
    asset_id: String,
    state: State<AppState>,
) -> Result<tauri::ipc::Response> {
    let (root, _) = roots(&state)?;
    let j = jobs::load(&root, &id)?;
    let asset = j["assets"]
        .as_array()
        .and_then(|a| a.iter().find(|a| a["id"] == asset_id))
        .ok_or_else(|| error("草稿媒体不存在"))?;
    let dir = jobs::dir(&root, &id)?.canonicalize()?;
    let rel = asset["path"]
        .as_str()
        .filter(|s| papers::safe_path(s))
        .ok_or_else(|| error("媒体路径无效"))?;
    let path = dir.join(rel).canonicalize()?;
    if !path.starts_with(&dir) || fs::metadata(&path)?.len() > 100 * 1024 * 1024 {
        return Err(error("媒体路径越界或过大"));
    }
    let bytes = fs::read(path)?;
    if papers::hash(&bytes) != asset["sha256"] {
        return Err(error("草稿媒体已损坏"));
    }
    Ok(tauri::ipc::Response::new(bytes))
}
#[tauri::command]
pub async fn cloud_test(provider: String, state: State<'_, AppState>) -> Result<Value> {
    let p = config(&state, &provider)?;
    let key = credentials::read_for_service(&provider)?;
    let started = std::time::Instant::now();
    let detail = match provider.as_str() {
        "deepseek" => {
            let v = cloud::chat(
                &p,
                &key,
                "Return JSON only: {\"ok\":true}",
                json!("Connection test. Return JSON."),
                false,
            )
            .await?;
            if v["ok"] != true {
                return Err(error("模型未通过 JSON 输出测试"));
            }
            "实际 JSON 模型请求成功"
        }
        "qwenTts" => {
            let bytes =
                speech::synthesize(&p, &key, "This is an IELTS Studio connection test.").await?;
            audio_type(&bytes)?;
            "已实际合成并检查音频格式"
        }
        "qwenAsr" => {
            let mut b = vec![0u8; 44 + 16000 * 2];
            b[..4].copy_from_slice(b"RIFF");
            let len = b.len() as u32 - 8;
            b[4..8].copy_from_slice(&len.to_le_bytes());
            b[8..12].copy_from_slice(b"WAVE");
            b[12..16].copy_from_slice(b"fmt ");
            b[16..20].copy_from_slice(&16u32.to_le_bytes());
            b[20..22].copy_from_slice(&1u16.to_le_bytes());
            b[22..24].copy_from_slice(&1u16.to_le_bytes());
            b[24..28].copy_from_slice(&16000u32.to_le_bytes());
            b[28..32].copy_from_slice(&32000u32.to_le_bytes());
            b[32..34].copy_from_slice(&2u16.to_le_bytes());
            b[34..36].copy_from_slice(&16u16.to_le_bytes());
            b[36..40].copy_from_slice(b"data");
            b[40..44].copy_from_slice(&32000u32.to_le_bytes());
            speech::recognize(&p, &key, &b).await?;
            "实际静音 WAV 请求成功；识别准确率仍需真实录音验证"
        }
        "evaluation" => {
            let v = cloud::chat(
                &p,
                &key,
                "Return JSON only: {\"ok\":true}",
                json!([{"type":"text","text":"Connection test, return JSON {ok:true}."}]),
                true,
            )
            .await?;
            if v["ok"] != true {
                return Err(error("音频模型未通过请求测试"));
            }
            "实际模型请求成功；音频评测能力需提交录音验证"
        }
        "mineru" => {
            let c = speech::client(&p)?;
            let r = c
                .get(speech::endpoint(
                    &p.base_url,
                    "extract-results/batch/00000000-0000-0000-0000-000000000000",
                )?)
                .bearer_auth(&key)
                .send()
                .await
                .map_err(|_| error("MinerU 连接失败"))?;
            let bytes = speech::bounded(r, 1024 * 1024).await?;
            let v: Value = serde_json::from_slice(&bytes)?;
            // A non-existent batch probes transport only; do not mistake it for authenticated parsing success.
            if v["code"] == "A0202" || v["code"] == "A0211" {
                return Err(error("MinerU Token 无效或过期"));
            }
            "已收到 MinerU 响应；批次探测不能证明解析权限，请用实际文件验证"
        }
        _ => return Err(error("未知服务")),
    };
    Ok(
        json!({"provider":provider,"detail":detail,"latencyMs":started.elapsed().as_millis(),"testedAt":now_ms(),"scope":"request_only"}),
    )
}

#[tauri::command]
pub async fn cloud_recheck(id: String, state: State<'_, AppState>) -> Result<Value> {
    let _lease = Lease::acquire(&id)?;
    let (root, _) = roots(&state)?;
    let mut j = jobs::load(&root, &id)?;
    papers::validate(&j["paper"])?;
    jobs::schedule_recheck(&mut j)?;
    jobs::save(&root, &j)?;
    Ok(j)
}

#[tauri::command]
pub fn cloud_progress(id: String, state: State<AppState>) -> Result<Value> {
    let root = job_root(&state, &id)?;
    let path = jobs::dir(&root, &id)?.join("status.json");
    if path.is_file() {
        if let Ok(v) = serde_json::from_slice::<Value>(&fs::read(&path)?) {
            return Ok(v);
        }
    }
    Ok(jobs::summary(&jobs::load(&root, &id)?))
}

#[tauri::command]
pub fn cloud_audio_start(id:String,state:State<AppState>)->Result<Value>{
 let _lease=Lease::acquire(&id)?;let (root,_)=roots(&state)?;let mut j=jobs::load(&root,&id)?;
 jobs::authorize_audio(&mut j)?;
 let p=config(&state,"qwenTts")?;credentials::read_for_service("qwenTts")?;
 let voices:Vec<_>=p.voice.split('|').map(str::trim).filter(|s|!s.is_empty()).collect();
 if voices.len()<3||voices[0]==voices[1]||voices[0]==voices[2]||voices[1]==voices[2]{return Err(error("请在设置中配置三种不同听力音色：音色A|音色B|旁白；TTS 尚未发送"));}
 jobs::save(&root,&j)?;Ok(j)
}
#[tauri::command]
pub fn cloud_retry_audio(id:String,state:State<AppState>)->Result<Value>{
 let _lease=Lease::acquire(&id)?;let(root,_)=roots(&state)?;let mut j=jobs::load(&root,&id)?;
 if j["kind"]!="generation"||j["state"]!="failed"||j["operation"]["phase"]!="audio" {return Err(error("仅可明确重发失败任务中结果未知的听力片段；普通下载失败请直接继续任务"));}
 let fingerprint=j["textFingerprint"].as_str().ok_or_else(||error("请先完成文字校审"))?;
 jobs::check_audio_authorization(&j,fingerprint)?;
 let dir=jobs::dir(&root,&id)?.join("audio-responses");let mut count=0;
 if dir.is_dir(){for entry in fs::read_dir(&dir)?{let entry=entry?;let name=entry.file_name().to_string_lossy().to_string();if let Some(hash)=name.strip_suffix(".attempt.json"){let raw=dir.join(format!("{hash}.audio"));if !raw.is_file()&&credentials::read_upload(&format!("generation-tts-{hash}"))?.is_none(){crate::source_library::audio_jobs::reset_response(&raw)?;count+=1;}}}}
 if count==0{return Err(error("没有结果未知的合成片段；已返回的音频及下载地址已保留，请直接继续任务"));}
 j["state"]=json!("pending");j["error"]=Value::Null;jobs::save(&root,&j)?;Ok(j)
}
#[tauri::command]
pub fn cloud_fix_text(id:String,state:State<AppState>)->Result<Value>{
 let _lease=Lease::acquire(&id)?;let (root,_)=roots(&state)?;let mut j=jobs::load(&root,&id)?;
 if j["kind"]!="generation"||!["review_required","failed","paused"].contains(&j["state"].as_str().unwrap_or(""))||!j["paper"].is_object()||j["stage"].as_u64().unwrap_or(0)<stages(&j["request"])?.len() as u64{return Err(error("请先打开已完成题目文字的待处理草稿；失败的单部分请使用重新整理当前部分"));}
 if j["humanEditedAt"].is_number(){return Err(error("此稿含人工修改，请先使用重新校审，避免自动重组覆盖你的修改"));}
 j["recheckOnly"]=json!(false);j["validation"]=Value::Null;j["textFingerprint"]=Value::Null;j["contentRepair"]=Value::Null;j["contentRepairCount"]=json!(0);j["audioAuthorization"]=Value::Null;j["textRepairRound"]=json!(uuid::Uuid::new_v4().to_string());j["state"]=json!("pending");j["error"]=Value::Null;
 jobs::save(&root,&j)?;Ok(j)
}
#[tauri::command]
pub fn cloud_retry_text(id:String,state:State<AppState>)->Result<Value>{
 let _lease=Lease::acquire(&id)?;let root=job_root(&state,&id)?;let mut j=jobs::load(&root,&id)?;if j["kind"]=="assessment"{jobs::retry_assessment(&root,&mut j)?;}else{jobs::retry_text(&root,&mut j)?;}jobs::save(&root,&j)?;Ok(j)
}
