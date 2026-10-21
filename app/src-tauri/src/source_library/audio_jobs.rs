use crate::{configuration::ProviderConfig,contracts::Result};
use serde_json::Value;
pub fn segment_key(text:&str,p:&ProviderConfig)->String{crate::papers::hash(serde_json::json!({"renderVersion":1,"text":text,"url":p.base_url,"model":p.model,"voice":p.voice,"language":"English"}).to_string().as_bytes())}
pub fn pending(d:&Value,done:&Value)->Result<Vec<Value>>{super::transcripts::validate(d)?;Ok(d["turns"].as_array().unwrap().iter().filter(|t|done[t["id"].as_str().unwrap()]["ready"]!=true).cloned().collect())}
pub fn cached(path:&std::path::Path)->bool{std::fs::read(path).is_ok_and(|b|crate::speech::validate_wav(&b).is_ok())}
pub fn unique_requests(keys:&[String],max:usize)->Vec<usize>{let mut seen=std::collections::HashSet::new();keys.iter().enumerate().filter_map(|(i,k)|seen.insert(k).then_some(i)).take(max).collect()}
pub async fn obtain<L,S,G,GF,D,DF>(load:L,mut save:S,mut generate:G,mut download:D)->Result<Vec<u8>>
where L:FnOnce()->Result<Option<crate::speech::TtsArtifact>>,S:FnMut(&crate::speech::TtsArtifact)->Result<()>,G:FnMut()->GF,GF:std::future::Future<Output=Result<crate::speech::TtsArtifact>>,D:FnMut(String)->DF,DF:std::future::Future<Output=Result<Vec<u8>>>{
 let artifact=match load()?{Some(a)=>a,None=>{let a=generate().await?;if matches!(a,crate::speech::TtsArtifact::Remote{..}){save(&a)?;}a}};
 match artifact{crate::speech::TtsArtifact::Inline(b)=>Ok(b),crate::speech::TtsArtifact::Remote{url}=>with_retry(||download(url.clone())).await}
}
pub async fn with_retry<F,Fut>(mut request:F)->Result<Vec<u8>> where F:FnMut()->Fut,Fut:std::future::Future<Output=Result<Vec<u8>>>{
 for attempt in 0..3 {match request().await {Ok(v)=>return Ok(v),Err(e)=>{let transient=e.message.contains("超时")||e.message.contains("HTTP 429")||e.message.contains("HTTP 5")||e.message.contains("读取云服务")||e.message.contains("下载考官音频失败");if !transient||attempt==2{return Err(e);}let specified=e.message.split("retry_after=").nth(1).and_then(|s|s.split_whitespace().next()).and_then(|s|s.parse::<u64>().ok());if specified.is_some_and(|n|n>60){return Err(e);}tokio::time::sleep(std::time::Duration::from_secs(specified.unwrap_or(2u64<<attempt))).await;}}}unreachable!()
}
/// Persist intent before synthesis and raw bytes before downstream decoding/metadata.
pub async fn obtain_durable<L,S,G,GF,D,DF>(path:&std::path::Path,load:L,mut save:S,mut generate:G,mut download:D)->Result<Vec<u8>>
where L:FnOnce()->Result<Option<crate::speech::TtsArtifact>>,S:FnMut(&crate::speech::TtsArtifact)->Result<()>,G:FnMut()->GF,GF:std::future::Future<Output=Result<crate::speech::TtsArtifact>>,D:FnMut(String)->DF,DF:std::future::Future<Output=Result<Vec<u8>>>{
 use std::{fs,io::Write};if path.is_file(){return Ok(fs::read(path)?);}if let Some(parent)=path.parent(){fs::create_dir_all(parent)?;}let marker=path.with_extension("attempt.json");
 let artifact=match load()?{Some(a)=>a,None=>{let mut intent=fs::OpenOptions::new().write(true).create_new(true).open(&marker).map_err(|_|crate::contracts::DesktopError::new("speech_uncertain","上次听力合成结果尚未确认，已停止自动重发。已完成片段保留；核对后可明确重新请求未完成片段（可能再次计费）。"))?;intent.write_all(b"{\"state\":\"sending\"}")?;intent.sync_all()?;drop(intent);let a=generate().await?;if matches!(a,crate::speech::TtsArtifact::Remote{..}){save(&a)?;}a}};
 let bytes=match artifact{crate::speech::TtsArtifact::Inline(b)=>b,crate::speech::TtsArtifact::Remote{url}=>with_retry(||download(url.clone())).await?};if bytes.is_empty()||bytes.len()>32*1024*1024{return Err(crate::contracts::DesktopError::new("audio_response","听力片段响应为空或超过32MB，未自动重新合成"));}
 let temp=path.with_extension(format!("pending-{}",uuid::Uuid::new_v4()));let mut file=fs::File::create(&temp)?;file.write_all(&bytes)?;file.sync_all()?;drop(file);if let Err(e)=fs::rename(&temp,path){if path.is_file(){fs::remove_file(&temp)?;return Ok(fs::read(path)?);}return Err(e.into());}crate::storage::atomic_json(&marker,&serde_json::json!({"state":"received"}))?;Ok(bytes)
}
pub fn reset_response(path:&std::path::Path)->Result<()>{for item in [path.to_path_buf(),path.with_extension("attempt.json")]{if item.is_file(){std::fs::rename(&item,item.with_extension(format!("previous-{}",uuid::Uuid::new_v4())))?;}}Ok(())}
