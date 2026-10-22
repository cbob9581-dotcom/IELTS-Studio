use crate::{cloud::{self,error,generation,jobs},configuration::ProviderConfig,contracts::{Result,now_ms},credentials,papers,speech,storage::atomic_json};
use serde_json::{json,Value};
use std::{fs,path::Path};
pub trait Providers {fn provider(&self,id:&str)->Result<(ProviderConfig,String)>;}

// Audio rendering units are independent of immutable source/evidence turns.
pub fn audio_turns(material:&Value)->Result<Vec<Value>>{
 let mut out=Vec::new();
 for turn in material["transcript"].as_array().ok_or_else(||error("听力缺少脚本"))?{
  let text=turn["text"].as_str().ok_or_else(||error("听力发言缺少正文"))?;
  for chunk in crate::source_library::transcripts::chunks(text,500){let mut clip=turn.clone();clip["text"]=json!(chunk);out.push(clip);}
 }Ok(out)
}
pub fn reuse_failed_review_for_repair(root:&Path,j:&mut Value)->Result<bool>{
 if j["kind"]!="generation"||j["humanEditedAt"].is_number()||j["validationProtocol"]!="demonstrated-v2"||j["validation"]["pass"]!=false||j["validation"]["issues"].as_array().is_none_or(|a|a.is_empty())||j["validationHash"]!=papers::hash(&serde_json::to_vec(&j["paper"])?){return Ok(false);}
 let dir=jobs::dir(root,j["id"].as_str().ok_or_else(||error("缺少任务编号"))?)?;let plan=generation::stages(&j["request"])?;let source=materials(&dir,&plan)?;
 if j["textFingerprint"]!=papers::hash(&serde_json::to_vec(&source)?){return Ok(false);}
 let mut verdict=generation::scope_verdict(&j["validation"],&source);let targets=cloud::review::targets(&verdict,&plan)?;verdict["requestRound"]=j["textRepairRound"].clone();atomic_json(&dir.join("content-repair-base.json"),&source)?;
 j["contentRepair"]=json!({"active":true,"next":0,"targets":targets,"verdict":verdict});j["contentRepairCount"]=json!(1);j["validation"]=Value::Null;j["audioAuthorization"]=Value::Null;Ok(true)
}

pub async fn step(services: &impl Providers, root: &Path, j: &mut Value) -> Result<()> {
    let plan = generation::stages(&j["request"])?;
    let stage = j["stage"].as_u64().unwrap_or(0) as usize;
    let dir = jobs::dir(root, j["id"].as_str().unwrap())?;
    if j["recheckOnly"]==true{
        papers::validate(&j["paper"])?;jobs::operation(root,j,"review","重新校审当前草稿（由后台任务管理，计入请求预算）",0,1,"份")?;
        let (p,key)=services.provider("deepseek")?;let context=json!({"paper":j["paper"],"sourceMaterials":materials(&dir,&plan)?,"requestedScope":j["request"],"repairRound":j["textRepairRound"]});let verdict=generation::scope_verdict(&cloud::review::audit(&p,&key,&dir,&context).await?,&context["sourceMaterials"]);
        j["validationHash"]=json!(papers::hash(&serde_json::to_vec(&j["paper"])?));j["validationProtocol"]=json!("demonstrated-v2");j["validationModel"]=json!(p.model);j["validationAt"]=json!(now_ms());j["validation"]=verdict;j["recheckOnly"]=json!(false);j["state"]=json!("review_required");return Ok(());
    }
    if stage < plan.len() {
        let (sec, part) = &plan[stage];
        let label = match sec.as_str() {
            "listening" => "听力",
            "reading" => "阅读",
            "writing" => "写作",
            _ => "口语",
        };
        jobs::operation(
            root,
            j,
            "questions",
            &format!("生成{label} Part {part}"),
            stage,
            plan.len(),
            "部分",
        )?;
        let (p,key)=services.provider("deepseek")?;
        let prompt = generation::prompt_for_request(
            sec,
            *part,
            j["request"]["kind"].as_str(),
            j["request"]["topic"].as_str().unwrap_or(""),
            &j["request"],
        )?;
        let path = dir.join(format!("part-{stage}.json"));
        let raw_path = dir.join(format!("part-{stage}-raw.json"));
        let restored=if raw_path.exists(){Some(serde_json::from_slice::<Value>(&fs::read(&raw_path)?)?)}else{None};
        let mut context=json!({"instruction":"Return the requested original part as JSON."});
        if sec=="speaking"&&*part==3 {
            if let Some(i)=plan.iter().position(|(s,p)|s=="speaking"&&*p==2){
                let previous:Value=serde_json::from_slice(&fs::read(dir.join(format!("part-{i}.json")))?)?;
                context["part2TopicCard"]=previous["questions"].clone();
            }
        }
        let v = if path.exists() {
            serde_json::from_slice(&fs::read(&path)?)?
        } else {
            let mut v = if let Some(v) = restored {
                v
            } else {
                let v = cloud::repair::chat_for_validation(
                    &p,
                    &key,
                    &prompt,
                    json!(serde_json::to_string(&context)?),
                    false,
                    Some(&dir.join(format!("part-{stage}.response.txt"))),
                )
                .await?;
                atomic_json(&raw_path, &v)?;
                v
            };
            let mut check=v.clone();
            if generation::normalize_generated(&mut check,sec,*part).and_then(|_|generation::validate_part(&check,sec,*part,j["request"]["kind"].as_str())).is_err(){
                jobs::operation(root,j,"repair","结果未通过校验，正在反馈具体原因并修复（最多1次）",stage,plan.len(),"部分")?;
            }
            v=cloud::repair::checked(&p,&key,&prompt,json!(serde_json::to_string(&context)?),false,&raw_path,v,|value|{
                generation::normalize_generated(value,sec,*part)?;
                generation::validate_part(value,sec,*part,j["request"]["kind"].as_str())
            }).await?;
            atomic_json(&path, &v)?;
            v
        };
        generation::validate_part(&v, sec, *part, j["request"]["kind"].as_str())?;
        j["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"section":sec,"part":part,"model":p.model,"completedAt":now_ms()}));
        j["stage"] = json!(stage + 1);
        j["operation"]["done"] = json!(stage + 1);
        return Ok(());
    }
    if j["contentRepair"]["active"]==true {
        let next=j["contentRepair"]["next"].as_u64().unwrap_or(0) as usize;
        let targets=j["contentRepair"]["targets"].as_array().ok_or_else(||error("文字修复目标记录缺失"))?.clone();
        if next<targets.len(){
            let index=targets[next].as_u64().ok_or_else(||error("文字修复目标无效"))? as usize;
            let (sec,part)=plan.get(index).ok_or_else(||error("文字修复范围已变化"))?;
            let base:Value=serde_json::from_slice(&fs::read(dir.join("content-repair-base.json"))?)?;
            let original=&base[index]["material"];
            jobs::operation(root,j,"content_fix",&format!("根据校审问题修复 {sec} Part {part}（仅文字，不调用 TTS）"),next,targets.len(),"部分")?;
            let (p,key)=services.provider("deepseek")?;
            let verdict=generation::scope_verdict(&j["contentRepair"]["verdict"],&materials(&dir,&plan)?);
            let fixed=cloud::review::correct(&p,&key,&dir,sec,*part,j["request"]["kind"].as_str(),original,&verdict,&j["request"]).await?;
            let file=dir.join(format!("part-{index}.json"));let backup=dir.join(format!("part-{index}-before-content-fix.json"));if !backup.exists(){fs::copy(&file,&backup)?;}
            if sec=="listening"&&original["transcript"]!=fixed["transcript"]{if let Some(assets)=j["assets"].as_array_mut(){assets.retain(|a|!a["id"].as_str().unwrap_or("").starts_with(&format!("listening-{part}-")));}j["audioAuthorization"]=Value::Null;}
            atomic_json(&file,&fixed)?;j["contentRepair"]["next"]=json!(next+1);j["validation"]=Value::Null;return Ok(());
        }
        j["contentRepair"]["active"]=json!(false);
    }
    // Every successful clip is checkpointed before advancing. A failed clip never regenerates previous clips.
    for (pi, (sec, part)) in plan.iter().enumerate() {
        let v: Value = serde_json::from_slice(&fs::read(dir.join(format!("part-{pi}.json")))?)?;
        let diagram_id = format!("diagram-{sec}-{part}");
        if v["questions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|q| q["kind"] == "diagram_label")
            && (j["visualHashes"][&diagram_id]!=papers::hash(&serde_json::to_vec(&v["visual"])? ) || j["assets"]
                .as_array()
                .is_none_or(|a| !a.iter().any(|a| a["id"] == diagram_id)))
        {
            let bytes = generation::diagram_png(&v["visual"])?;
            add_asset(
                j,
                &dir,
                &diagram_id,
                &format!("media/{diagram_id}.png"),
                "image/png",
                &bytes,
            )?;
            if !j["visualHashes"].is_object(){j["visualHashes"]=json!({});}
            j["visualHashes"][&diagram_id]=json!(papers::hash(&serde_json::to_vec(&v["visual"])?));
            jobs::save(root, j)?;
            return Ok(());
        }
        if sec == "writing"
            && *part == 1
            && (j["visualHashes"]["writing-table"]!=papers::hash(&serde_json::to_vec(&v["visual"])? ) || j["assets"]
                .as_array()
                .is_none_or(|a| !a.iter().any(|a| a["id"] == "writing-table")))
        {
            let bytes = cloud::visuals::render(&v["visual"])?;
            add_asset(
                j,
                &dir,
                "writing-table",
                "media/writing-table.png",
                "image/png",
                &bytes,
            )?;
            if !j["visualHashes"].is_object(){j["visualHashes"]=json!({});}
            j["visualHashes"]["writing-table"]=json!(papers::hash(&serde_json::to_vec(&v["visual"])?));
            jobs::save(root, j)?;
            return Ok(());
        }
    }
    let material=materials(&dir,&plan)?;
    let fingerprint=papers::hash(&serde_json::to_vec(&material)?);
    let mut assembled=assemble(j,&dir,&plan)?;
    papers::validate(&assembled)?;
    j["listeningScripts"]=json!(material.as_array().unwrap().iter().filter(|m|m["section"]=="listening").map(|m|json!({"part":m["part"],"turns":m["material"]["transcript"]})).collect::<Vec<_>>());
    if j["textFingerprint"]!=fingerprint||j["validation"].is_null()||j["validationProtocol"]!="demonstrated-v2"{
        jobs::operation(root,j,"review","核对题目、答案、图表及依据（仅文字，TTS 未启动）",0,1,"次")?;
        let (p,key)=services.provider("deepseek")?;
        j["paper"]=assembled.clone();jobs::save(root,j)?;
        let context=json!({"paper":assembled,"sourceMaterials":material,"requestedScope":j["request"],"repairRound":j["textRepairRound"]});
        let verdict=cloud::review::audit(&p,&key,&dir,&context).await?;
        j["validationProtocol"]=json!("demonstrated-v2");j["validation"]=verdict;j["validationModel"]=json!(p.model);j["textFingerprint"]=json!(fingerprint);
        j["paper"]=assembled.clone();j["validationHash"]=json!(papers::hash(&serde_json::to_vec(&j["paper"])?));
    }
    // Apply hard scope rules even when the semantic audit was restored from cache.
    j["validation"]=generation::scope_verdict(&j["validation"],&material);
    if j["validation"]["pass"]!=true {
        let limit=crate::task_policy::TaskPolicy::load(root)?.max_content_fix_rounds as u64;
        if j["contentRepairCount"].as_u64().unwrap_or(0)<limit {
            let targets=cloud::review::targets(&j["validation"],&plan)?;
            atomic_json(&dir.join("content-repair-base.json"),&material)?;
            j["contentRepairCount"]=json!(j["contentRepairCount"].as_u64().unwrap_or(0)+1);j["contentRepair"]=json!({"active":true,"next":0,"targets":targets,"verdict":j["validation"]});
            j["contentRepair"]["verdict"]["requestRound"]=j["textRepairRound"].clone();
            jobs::operation(root,j,"content_fix","校审发现具体问题，将修复对应部分并重新核对（仅文字）",0,targets.len(),"部分")?;return Ok(());
        }
        j["state"]=json!("review_required");j["error"]=json!(format!("已完成 {limit} 轮限定的文字修复并复查，仍有下列具体问题。成功部分和草稿已保存，TTS 未启动。"));
        jobs::operation(root,j,"review_blocked","文字草稿需修复；语音合成尚未启动",0,1,"份")?;return Ok(());
    }
    let mut expected=0;let mut completed=0;
    for (i,(_,part)) in plan.iter().enumerate().filter(|(_, (s,_))|s=="listening"){
        let turns=audio_turns(&material[i]["material"])?;expected+=turns.len();
        completed+=turns.iter().enumerate().filter(|(ti,_)|j["assets"].as_array().is_some_and(|a|a.iter().any(|a|a["id"]==format!("listening-{part}-{ti}")))).count();
    }
    j["audioPending"]=json!(completed<expected);j["audioSegmentCount"]=json!(expected);j["audioCompleted"]=json!(completed);
    if completed<expected&&jobs::check_audio_authorization(j,&fingerprint).is_err(){
        j["state"]=json!("review_required");
        jobs::operation(root,j,"audio_choice","文字校审已通过；请选择上传音频或单独启动 TTS",completed,expected,"段音频")?;return Ok(());
    }
    if completed<expected { for (pi,(sec,part)) in plan.iter().enumerate(){
        let v=&material[pi]["material"];
        if sec == "listening" {
            jobs::check_audio_authorization(j,&fingerprint)?;
            let (p,key)=services.provider("qwenTts")?;
            let voices: Vec<String> = p
                .voice
                .split('|')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if voices.len() < 3
                || voices[0] == voices[2]
                || voices[1] == voices[2]
                || ((*part == 1 || *part == 3) && voices[0] == voices[1])
            {
                return Err(error(
                    "听力需要独立对话与旁白音色，配置格式：音色A|音色B|旁白",
                ));
            }
            let transcript = audio_turns(v)?;
            let mut pending = Vec::new();
            let mut completed = 0;
            for (ti, t) in transcript.iter().enumerate() {
                let id = format!("listening-{part}-{ti}");
                if j["assets"]
                    .as_array()
                    .is_some_and(|a| a.iter().any(|a| a["id"] == id))
                {
                    completed += 1;
                    continue;
                }
                if pending.len() < crate::task_policy::TaskPolicy::load(root)?.tts_concurrency {
                    pending.push((id, t.clone()));
                }
            }
            if !pending.is_empty() {
                jobs::operation(
                    root,
                    j,
                    "audio",
                    &format!("制作听力 Part {part} 录音（本批 {} 条）",pending.len()),
                    completed,
                    transcript.len(),
                    "条",
                )?;
                let mut tasks = tokio::task::JoinSet::new();
                for (id, t) in pending {
                    let mut p = p.clone();
                    let key = key.clone();
                    let response_dir=dir.join("audio-responses");
                    p.voice = voices[t["speaker"].as_u64().unwrap() as usize].clone();
                    tasks.spawn(async move {
                        let cache=crate::source_library::audio_jobs::segment_key(t["text"].as_str().unwrap(),&p);
                        let ticket=format!("generation-tts-{cache}");
                        let result=crate::source_library::audio_jobs::obtain_durable(&response_dir.join(format!("{cache}.audio")),||{credentials::read_upload(&ticket)?.map(|s|serde_json::from_str(&s).map_err(Into::into)).transpose()},|a|credentials::save_upload(&ticket,&serde_json::to_string(a)?),||speech::tts_generate(&p,&key,t["text"].as_str().unwrap()),|url|{let p=p.clone();async move{speech::tts_download(&p,&url).await}}).await;
                        (id,ticket,result)
                    });
                }
                let mut failure = None;
                while let Some(done) = tasks.join_next().await {
                    let (id, ticket, result) =
                        done.map_err(|_| error("录音任务中断；已完成的条目已保存"))?;
                    match result {
                        Ok(bytes) => {
                            let (ext, mime) = audio_type(&bytes)?;
                            add_asset(j, &dir, &id, &format!("media/{id}.{ext}"), mime, &bytes)?;
                            credentials::clear_upload(&ticket)?;
                            completed += 1;
                            j["operation"]["done"] = json!(completed);
                            jobs::save(root, j)?;
                        }
                        Err(e) => {
                            failure = Some(e);
                        }
                    }
                }
                if let Some(e) = failure {
                    return Err(e);
                }
                return Ok(());
            }
        }
    }
    }
    // Only media changes after the text audit. Preserve any reviewed user edits.
    assembled=assemble(j,&dir,&plan)?;
    if !jobs::content_approved(j){return Err(error("当前草稿的文字校审已失效；语音合成不会继续"));}
    for target in j["paper"]["sections"].as_array_mut().unwrap(){
        if target["section"]=="listening"{if let Some(source)=assembled["sections"].as_array().unwrap().iter().find(|s|s["section"]=="listening"){target["audioAssetIds"]=source["audioAssetIds"].clone();target["audioParts"]=source["audioParts"].clone();}}
    }
    j["paper"]["assets"]=assembled["assets"].clone();papers::validate(&j["paper"])?;
    j["validationHash"]=json!(papers::hash(&serde_json::to_vec(&j["paper"])?));
    j["audioPending"]=json!(false);j["state"]=json!("review_required");j["error"]=Value::Null;
    jobs::operation(root,j,"ready","题目和所需媒体已准备完成，核对后发布",1,1,"份")?;
    Ok(())
}
pub fn materials(dir:&Path,plan:&[(String,u64)])->Result<Value>{
 let mut out=Vec::new();for (i,(section,part)) in plan.iter().enumerate(){let material:Value=serde_json::from_slice(&fs::read(dir.join(format!("part-{i}.json")))?)?;out.push(json!({"section":section,"part":part,"material":material}));}Ok(json!(out))
}
pub fn audio_type(b: &[u8]) -> Result<(&'static str, &'static str)> {
    if b.starts_with(b"RIFF") && b.get(8..12) == Some(b"WAVE") {
        Ok(("wav", "audio/wav"))
    } else if b.starts_with(b"ID3") || (b.len() > 2 && b[0] == 255 && b[1] & 224 == 224) {
        Ok(("mp3", "audio/mpeg"))
    } else {
        Err(error("TTS 未返回可识别的 WAV/MP3 音频"))
    }
}
fn assemble(j: &Value, dir: &Path, plan: &[(String, u64)]) -> Result<Value> {
    let mut sections = Vec::new();
    for sec in ["listening", "reading", "writing", "speaking"] {
        let mut qs = Vec::new();
        let mut paras = Vec::new();
        let mut audios = Vec::new();
        let mut audio_parts = serde_json::Map::new();
        for (i, (s, p)) in plan.iter().enumerate().filter(|(_, (s, _))| s == sec) {
            let v: Value = serde_json::from_slice(&fs::read(dir.join(format!("part-{i}.json")))?)?;
            for (qi, q) in v["questions"].as_array().unwrap().iter().enumerate() {
                let mut q = q.clone();
                q["id"] = json!(format!("{sec}-{p}-{}", qi + 1));
                let number = if ["listening", "reading"].contains(&sec) {
                    qs.iter()
                        .map(|q: &Value| q["points"].as_u64().unwrap_or(1))
                        .sum::<u64>()
                        + 1
                } else {
                    qs.len() as u64 + 1
                };
                q["number"] = json!(number);
                if q["kind"] == "diagram_label" {
                    q["imageAssetId"] = json!(format!("diagram-{sec}-{p}"));
                }
                if sec == "writing" && *p == 1 {
                    q["imageAssetId"] = json!("writing-table");
                }
                qs.push(q);
            }
            if sec != "listening" {
                for (n, pv) in v["paragraphs"].as_array().unwrap().iter().enumerate() {
                    let mut pv = pv.clone();
                    pv["id"] = json!(format!("{sec}-{p}-p{n}"));
                    paras.push(pv);
                }
            }
            if sec == "listening" {
                for ti in 0..audio_turns(&v)?.len() {
                    let id = format!("listening-{p}-{ti}");
                    if j["assets"].as_array().is_some_and(|a|a.iter().any(|a|a["id"]==id)) {audio_parts.insert(id.clone(), json!(p));audios.push(id);}
                }
            }
            let _ = s;
        }
        if qs.is_empty() {
            continue;
        }
        let duration = match sec {
            "listening" => 30 * 60 * 1000,
            "reading" | "writing" => 60 * 60 * 1000,
            _ => 14 * 60 * 1000,
        };
        let mut section = json!({"section":sec,"title":format!("Academic {sec}"),"durationMs":duration,"timingSource":"https://ielts.org/take-a-test/test-types/ielts-academic-test ; AI practice timing, not calibrated official audio","paragraphs":paras,"questions":qs});
        if sec == "listening" {
            section["audioAssetIds"] = json!(audios);
            section["audioParts"] = Value::Object(audio_parts);
        }
        sections.push(section);
    }
    Ok(
        json!({"schemaVersion":1,"id":format!("ai-{}",j["id"].as_str().unwrap()),"version":1,"title":format!("Academic 原创模拟 · {}",j["request"]["topic"].as_str().filter(|s|!s.is_empty()).unwrap_or("综合主题")),"testType":"academic","status":"review_required","provenance":{"kind":"ai_generated","source":format!("AI original simulation; official IELTS task formats; job {}. Not an authentic official paper.",j["id"].as_str().unwrap())},"sections":sections,"assets":j["assets"].as_array().cloned().unwrap_or_default()}),
    )
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

#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 fn explicit_fix_reuses_only_a_failed_verdict_bound_to_the_current_paper_and_sources(){
  let root=tempfile::tempdir().unwrap();let mut j=jobs::create(root.path(),"generation",json!({"section":"listening","part":2})).unwrap();let dir=jobs::dir(root.path(),j["id"].as_str().unwrap()).unwrap();atomic_json(&dir.join("part-0.json"),&json!({"paragraphs":[],"questions":[],"transcript":[]})).unwrap();let plan=generation::stages(&j["request"]).unwrap();j["paper"]=json!({"title":"unchanged"});j["validation"]=json!({"pass":false,"issues":[{"questionId":"listening-2-1","reason":"repair shared options"}]});j["validationProtocol"]=json!("demonstrated-v2");j["validationHash"]=json!(papers::hash(&serde_json::to_vec(&j["paper"]).unwrap()));j["textFingerprint"]=json!(papers::hash(&serde_json::to_vec(&materials(&dir,&plan).unwrap()).unwrap()));
  let mut changed=j.clone();changed["paper"]["title"]=json!("changed after audit");assert!(!reuse_failed_review_for_repair(root.path(),&mut changed).unwrap());assert!(reuse_failed_review_for_repair(root.path(),&mut j).unwrap());assert_eq!(j["contentRepair"]["targets"],json!([0]));assert_eq!(j["contentRepairCount"],1);assert!(j["validation"].is_null());
 }
 #[tokio::test]
 async fn cached_full_reading_pass_cannot_bypass_machine_word_range_or_issue_new_requests(){
  let root=tempfile::tempdir().unwrap();let mut j=jobs::create(root.path(),"generation",json!({"section":"reading"})).unwrap();let dir=jobs::dir(root.path(),j["id"].as_str().unwrap()).unwrap();
  for part in 1..=3{let qs:Vec<_>=(1..=generation::count("reading",part)).map(|n|json!({"id":format!("q{n}"),"number":n,"part":part,"kind":"gap_fill","points":1,"prompt":"Write ONE WORD: ____ trees","answer":{"accepted":["oak"],"caseSensitive":false,"maxWords":1},"evidence":["oak and pine"]})).collect();atomic_json(&dir.join(format!("part-{}.json",part-1)),&json!({"paragraphs":[{"id":"p1","part":part,"text":"oak and pine ".repeat(200)}],"questions":qs,"transcript":[]})).unwrap();}
  let plan=generation::stages(&j["request"]).unwrap();j["stage"]=json!(3);j["validationProtocol"]=json!("demonstrated-v2");j["paper"]=assemble(&j,&dir,&plan).unwrap();j["validation"]=json!({"pass":true,"issues":[]});j["textFingerprint"]=json!(papers::hash(&serde_json::to_vec(&materials(&dir,&plan).unwrap()).unwrap()));j["validationHash"]=json!(papers::hash(&serde_json::to_vec(&j["paper"]).unwrap()));j["contentRepairCount"]=json!(2);
  step(&NoProviders,root.path(),&mut j).await.unwrap();assert_eq!(j["validation"]["pass"],false);assert_eq!(j["operation"]["phase"],"review_blocked");assert!(!jobs::content_approved(&j));
 }
 #[test]
 fn audio_clip_boundaries_preserve_source_evidence_and_speaker_order(){
  let text="The original lecture contains a long sentence about oak trees. ".repeat(35);
  let material=json!({"transcript":[{"id":"t1","speaker":2,"text":"Read the questions."},{"id":"t2","speaker":0,"text":text},{"id":"t3","speaker":2,"text":"Check your answers."}]});
  let before=material.clone();let clips=audio_turns(&material).unwrap();assert_eq!(material,before);assert!(clips.len()>3);assert!(clips.iter().all(|c|c["text"].as_str().unwrap().chars().count()<=500));assert_eq!(clips[0]["speaker"],2);assert_eq!(clips.last().unwrap()["speaker"],2);
  let spoken=clips.iter().filter(|c|c["speaker"]==0).map(|c|c["text"].as_str().unwrap()).collect::<Vec<_>>().join(" ");assert_eq!(spoken.split_whitespace().collect::<Vec<_>>(),text.split_whitespace().collect::<Vec<_>>());
 }
 struct NoProviders;
 impl Providers for NoProviders {fn provider(&self,id:&str)->Result<(ProviderConfig,String)>{panic!("no provider should be accessed before explicit audio action: {id}")}}
 #[tokio::test]
 async fn approved_listening_text_stops_before_even_reading_tts_configuration(){
  let root=tempfile::tempdir().unwrap();let mut j=jobs::create(root.path(),"generation",json!({"section":"listening","part":1})).unwrap();
  let dir=jobs::dir(root.path(),j["id"].as_str().unwrap()).unwrap();
  let qs:Vec<_>=(1..=10).map(|n|json!({"id":format!("q{n}"),"number":n,"part":1,"kind":"gap_fill","prompt":"Write ONE WORD: ____ trees","points":1,"evidence":["oak and pine"],"answer":{"accepted":["oak"],"caseSensitive":false,"maxWords":1}})).collect();
  let material=json!({"paragraphs":[],"questions":qs,"transcript":[{"speaker":2,"text":"Read the questions."},{"speaker":0,"text":"We will plant oak and pine in the new park."},{"speaker":1,"text":"The trees will be planted next month."}]});
  atomic_json(&dir.join("part-0.json"),&material).unwrap();let plan=generation::stages(&j["request"]).unwrap();j["stage"]=json!(1);
  j["validationProtocol"]=json!("demonstrated-v2");j["paper"]=assemble(&j,&dir,&plan).unwrap();j["validation"]=json!({"pass":true,"issues":[]});j["textFingerprint"]=json!(papers::hash(&serde_json::to_vec(&materials(&dir,&plan).unwrap()).unwrap()));j["validationHash"]=json!(papers::hash(&serde_json::to_vec(&j["paper"]).unwrap()));
  step(&NoProviders,root.path(),&mut j).await.unwrap();assert_eq!(j["state"],"review_required");assert_eq!(j["audioPending"],true);assert_eq!(j["audioSegmentCount"],3);assert!(j.get("audioAuthorization").is_none());assert!(jobs::content_approved(&j));
 }
}
