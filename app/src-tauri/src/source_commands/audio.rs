use super::*;
use lib::{transcripts,audio_assets as assets,audio_jobs};
use crate::{configuration::ProviderConfig,speech};

pub(super) fn action(a:&str,r:&Path,p:&mut Value,v:&Value,state:&State<AppState>)->Result<bool>{
 let target=v["targetId"].as_str().unwrap_or("");
 if !matches!(a,"transcript_extract"|"transcript_import"|"transcript_save"|"audio_start"|"audio_verify"|"audio_reset_failed"){return Ok(false);}
 if !p["audioDrafts"].is_array(){p["audioDrafts"]=json!([]);}
 match a {
  "transcript_extract"=>{
   let u=find(p,"units",target)?.clone();if u["role"]!="transcript"{return Err(err("请选择听力原文题组"));}
   if p["audioDrafts"].as_array().unwrap().iter().any(|d|d["sourceUnitId"]==target){return Err(err("此题组已有听力稿，请从已保存稿件继续编辑"));}
   let b=lib::blocks(r,p,u["documentId"].as_str().unwrap())?;let start=u["start"].as_u64().unwrap() as usize;let end=u["end"].as_u64().unwrap() as usize;
   let mut d=transcripts::from_blocks(b.get(start..=end).ok_or_else(||err("听力稿范围越界"))?,u["test"].as_str().unwrap(),u["part"].as_u64().unwrap())?;
   d["sourceUnitId"]=json!(target);d["documentId"]=u["documentId"].clone();p["audioDrafts"].as_array_mut().unwrap().push(d);
  },
  "transcript_import"=>{
   let text=if let Some(path)=v["path"].as_str(){let path=Path::new(path);if !matches!(path.extension().and_then(|s|s.to_str()),Some("txt"|"md"))||fs::metadata(path)?.len()>200000{return Err(err("用户听力稿支持200KB以内的 UTF-8 TXT/MD"));}fs::read_to_string(path).map_err(|_|err("请将听力稿保存为 UTF-8 文本"))?}else{v["text"].as_str().unwrap_or("").into()};
   let d=transcripts::parse(&text,"user",v["test"].as_str().unwrap_or("Test 1"),v["part"].as_u64().unwrap_or(1))?;p["audioDrafts"].as_array_mut().unwrap().push(d);
  },
  "transcript_save"=>{
   let d=find_mut(p,"audioDrafts",target)?;let rev=d["revision"].as_u64().unwrap_or(0);if v["draftRevision"]!=rev{return Err(err("听力稿已更新，请刷新后重试；没有覆盖其他修改"));}
   let mut next=d.clone();for k in ["turns","events","voices","test","part"]{next[k]=v["draft"][k].clone();}
   let parallelism=v["draft"]["parallelism"].as_u64().unwrap_or(3);if !(1..=3).contains(&parallelism){return Err(err("语音生成并发数须在1–3之间"));}next["parallelism"]=json!(parallelism);
   if next["test"].as_str().is_none_or(|s|s.trim().is_empty()||s.len()>100)||!(1..=4).contains(&next["part"].as_u64().unwrap_or(0)){return Err(err("请选择有效 Test 和 Part"));}
   transcripts::validate(&next)?;next["revision"]=json!(rev+1);next["state"]=json!("draft");next["job"]=Value::Null;next["error"]=Value::Null;*d=next;
  },
  "audio_start"=>{
   let d=find_mut(p,"audioDrafts",target)?;transcripts::validate(d)?;
   let provider=cloud_commands::config(state,"qwenTts")?;
   for t in d["turns"].as_array().unwrap(){let name=t["speaker"].as_str().unwrap();if d["voices"][name].as_str().is_none_or(|s|s.trim().is_empty()||s.contains('|')||s.len()>100){return Err(err(format!("请为角色 {name} 填写一个有效音色，然后保存听力稿")));}}
   if d["state"]!="complete"{if d["job"].is_null()||d["state"]=="ready"{let total=if v["preview"]==true{d["turns"].as_array().unwrap().len().min(3)}else{d["turns"].as_array().unwrap().len()};d["job"]=json!({"provider":provider,"segments":{},"preview":v["preview"]==true,"done":0,"total":total});}d["state"]=json!("pending");d["error"]=Value::Null;}
   if d["state"]!="complete"{d["job"]["provider"]=json!(provider);}
  },
  "audio_reset_failed"=>{
   let d=find(p,"audioDrafts",target)?;let provider:ProviderConfig=serde_json::from_value(d["job"]["provider"].clone())?;let base=lib::dir(r,p["id"].as_str().unwrap())?.join("audio-cache");
   for t in d["turns"].as_array().unwrap(){let mut cfg=provider.clone();cfg.voice=d["voices"][t["speaker"].as_str().unwrap()].as_str().unwrap().into();let hash=audio_jobs::segment_key(t["text"].as_str().unwrap(),&cfg);if !audio_jobs::cached(&base.join(format!("{hash}.wav"))){credentials::clear_upload(&format!("tts-{hash}"))?;audio_jobs::reset_response(&base.join(format!("{hash}.response")))?;}}
  },
  "audio_verify"=>{
   let doc=find(p,"documents",target)?.clone();if doc["state"]!="media"{return Err(err("请选择听力音频"));}let path=lib::document_path(r,p,&doc)?;let bytes=fs::read(path)?;let duration=assets::duration(&bytes,doc["extension"].as_str().unwrap_or("wav"))?;let d=find_mut(p,"documents",target)?;d["durationMs"]=json!(duration);d["audioVerified"]=json!(true);
  },_=>unreachable!()
 }Ok(true)
}
fn checkpoint(r:&Path,p:&mut Value)->Result<()>{let rev=p["revision"].as_u64().unwrap();lib::save(r,p,rev)}
fn write_file(path:&Path,bytes:&[u8])->Result<()> {fs::create_dir_all(path.parent().unwrap())?;let temp=path.with_extension(format!("{}.tmp",lib::id()));fs::write(&temp,bytes)?;if path.exists(){fs::remove_file(path)?;}fs::rename(temp,path)?;Ok(())}
pub(super) async fn step(r:&Path,p:&mut Value,target:&str,state:&State<'_,AppState>)->Result<()> {
 let result=work(r,p,target,state).await;
 if let Err(e)=result {let d=find_mut(p,"audioDrafts",target)?;d["state"]=json!("failed");d["error"]=json!(format!("{} Part {}：{}。已完成片段保留，可修正设置后继续。",d["test"].as_str().unwrap_or(""),d["part"],e.message));checkpoint(r,p)?;}
 Ok(())
}
async fn work(r:&Path,p:&mut Value,target:&str,_state:&State<'_,AppState>)->Result<()> {
 let mut d=find(p,"audioDrafts",target)?.clone();if d["state"]=="complete"{return Ok(());}if d["job"].is_null(){return Err(err("请先确认角色音色并开始生成"));}transcripts::validate(&d)?;
 let preview=d["job"]["preview"]==true;
 if preview{let turns=d["turns"].as_array().unwrap().iter().take(3).cloned().collect::<Vec<_>>();let last=turns.last().unwrap()["id"].clone();let end=d["events"].as_array().unwrap().iter().position(|e|e["type"]=="speech"&&e["turnId"]==last).unwrap();d["events"]=json!(&d["events"].as_array().unwrap()[..=end]);d["turns"]=json!(turns);}
 let provider:ProviderConfig=serde_json::from_value(d["job"]["provider"].clone())?;
 let base=lib::dir(r,p["id"].as_str().unwrap())?;let cache=base.join("audio-cache");fs::create_dir_all(&cache)?;
 let mut missing=vec![];let mut registered=serde_json::Map::new();
 for t in d["turns"].as_array().unwrap(){let mut cfg=provider.clone();cfg.voice=d["voices"][t["speaker"].as_str().unwrap()].as_str().unwrap().into();let key=audio_jobs::segment_key(t["text"].as_str().unwrap(),&cfg);let path=cache.join(format!("{key}.wav"));if audio_jobs::cached(&path){registered.insert(t["id"].as_str().unwrap().into(),json!({"key":key,"ready":true}));}else{missing.push((t.clone(),cfg,key));}}
 {let draft=find_mut(p,"audioDrafts",target)?;draft["job"]["segments"]=json!(registered);draft["job"]["done"]=json!(registered.len());draft["state"]=json!(if missing.is_empty(){"assembling"}else{"synthesizing"});draft["error"]=Value::Null;}checkpoint(r,p)?;
 if !missing.is_empty(){
  let key=credentials::read_for_service("qwenTts")?;let mut set=tokio::task::JoinSet::new();
  let selected=audio_jobs::unique_requests(&missing.iter().map(|(_,_,h)|h.clone()).collect::<Vec<_>>(),(d["parallelism"].as_u64().unwrap_or(3) as usize).min(crate::task_policy::TaskPolicy::load(r)?.tts_concurrency));
  for i in selected {let (t,cfg,hash)=missing[i].clone();let key=key.clone();let cache=cache.clone();set.spawn(async move{let result=async {
    let ticket=format!("tts-{hash}");
    let bytes=audio_jobs::obtain_durable(&cache.join(format!("{hash}.response")),||{credentials::read_upload(&ticket)?.map(|s|serde_json::from_str(&s).map_err(Into::into)).transpose()},|a|credentials::save_upload(&ticket,&serde_json::to_string(a)?),||speech::tts_generate(&cfg,&key,t["text"].as_str().unwrap()),|url|{let cfg=cfg.clone();async move{speech::tts_download(&cfg,&url).await}}).await?;
    let pcm=tokio::task::spawn_blocking(move||assets::decode(&bytes,"wav",300)).await.map_err(|_|err("语音解码线程异常"))??;
    let bytes=assets::wav(&pcm,24000);write_file(&cache.join(format!("{hash}.wav")),&bytes)?;credentials::clear_upload(&ticket)?;Ok::<_,crate::contracts::DesktopError>(json!({"key":hash,"ready":true,"durationMs":pcm.len()/24}))
   }.await;(t["id"].as_str().unwrap().to_string(),result)});}
  let mut failures=vec![];
  while let Some(result)=set.join_next().await {match result {Ok((tid,Ok(meta)))=>{let draft=find_mut(p,"audioDrafts",target)?;draft["job"]["segments"][&tid]=meta;let done=draft["job"]["segments"].as_object().unwrap().len();draft["job"]["done"]=json!(done);checkpoint(r,p)?;},Ok((tid,Err(e)))=>failures.push(format!("正文 {tid} 生成失败：{}",e.message)),Err(_)=>failures.push("语音请求线程异常".into())}}
  if !failures.is_empty(){return Err(err(failures.join("；")));}return Ok(());
 }
 let events=d["events"].as_array().unwrap().clone();let turns=d["turns"].as_array().unwrap().clone();
 let cached=registered;let render=tokio::task::spawn_blocking(move||{
  let mut segments=std::collections::HashMap::new();for t in turns {let tid=t["id"].as_str().unwrap();let key=cached[tid]["key"].as_str().unwrap();segments.insert(tid.into(),assets::decode(&fs::read(cache.join(format!("{key}.wav")))?,"wav",300)?);}let samples=assets::render(&events,&segments)?;Ok::<_,crate::contracts::DesktopError>((assets::wav(&samples,24000),samples.len()/24))
 }).await.map_err(|_|err("音轨组装线程异常"))??;
 let hash=papers::hash(&render.0);let aid=format!("audio-{}-{}-{}-{}",target,d["revision"].as_u64().unwrap_or(0),if preview{"preview"}else{"full"},&hash[..12]);let rel=format!("media/{aid}.wav");write_file(&base.join(&rel),&render.0)?;
 if !p["documents"].as_array().unwrap().iter().any(|doc|doc["id"]==aid){
  p["documents"].as_array_mut().unwrap().push(json!({"id":aid,"name":format!("{} Part {} · {}{}",d["test"].as_str().unwrap(),d["part"],if d["origin"]=="book"{"真题脚本合成"}else{"用户文本合成"},if preview{"（前3段试听）"}else{""}),"path":rel,"extension":"wav","sha256":hash,"roles":["audio"],"state":"media","scanCursor":0,"pageCount":0,"blockCount":0,"audioVerified":true,"preview":preview,"durationMs":render.1,"origin":d["origin"],"draftId":target,"draftRevision":d["revision"],"transcript":d["turns"],"sourceRefs":d["sourceRefs"],"binding":{"test":d["test"],"section":"listening","scope":"part","part":d["part"],"active":false,"order":d["part"].as_u64().unwrap()*100}}));
 }
 let draft=find_mut(p,"audioDrafts",target)?;draft["state"]=json!(if preview{"ready"}else{"complete"});draft[if preview{"previewAssetId"}else{"assetId"}]=json!(aid);draft["durationMs"]=json!(render.1);checkpoint(r,p)?;Ok(())
}
