//! The learner receives a usable paper; authoring confirmation is not an exam step.
use crate::{contracts::{Result,now_ms},papers,storage::Store};
use serde_json::{Value,json};
use std::path::Path;
use super::jobs;
pub fn validate_audio_upload(bytes:&[u8],extension:&str)->Result<u64>{crate::source_library::audio_assets::duration(bytes,extension)}
pub fn ready(j:&Value)->bool{
 j["kind"]=="generation"&&j["state"]=="review_required"&&j["audioPending"]!=true&&jobs::content_approved(j)
 &&j["paper"]["sections"].as_array().into_iter().flatten().all(|s|s["section"]!="listening"||s["audioAssetIds"].as_array().is_some_and(|a|!a.is_empty()))
}
pub fn attach_audio(j:&mut Value,asset_id:&str)->Result<()>{
 if j["kind"]!="generation"||j["state"]!="review_required"||!jobs::content_approved(j){return Err(super::error("题目自动检查尚未完成，未绑定音频"));}
 if !j["assets"].as_array().into_iter().flatten().any(|a|a["id"]==asset_id&&a["mediaType"].as_str().is_some_and(|m|m.starts_with("audio/"))){return Err(super::error("尚未登记所选音频"));}
 let part=j["request"]["part"].as_u64();let s=j["paper"]["sections"].as_array_mut().and_then(|s|s.iter_mut().find(|s|s["section"]=="listening")).ok_or_else(||super::error("此卷没有听力部分"))?;
 s["audioAssetIds"]=json!([asset_id]);s["audioPlaylist"]=json!([{"assetId":asset_id,"part":part,"origin":"uploaded","segments":[]}]);s["audioParts"]=json!({});if let Some(part)=part{s["audioParts"][asset_id]=json!(part);}
 j["paper"]["assets"]=j["assets"].clone();j["audioPending"]=json!(false);j["validationHash"]=json!(papers::hash(&serde_json::to_vec(&j["paper"])?));Ok(())
}
pub fn finish(store:&Store,root:&Path,j:&mut Value)->Result<bool>{
 if !ready(j){return Ok(false);}let id=j["id"].as_str().ok_or_else(||super::error("缺少任务编号"))?.to_owned();let mut paper=j["paper"].clone();paper["status"]=json!("ready");papers::validate(&paper)?;
 j["paper"]=papers::install(store,&paper,&jobs::dir(root,&id)?)?;j["state"]=json!("complete");j["error"]=Value::Null;j["publishedAt"]=json!(now_ms());jobs::operation(root,j,"ready","试卷已准备完成，可以进入考场",1,1,"份")?;jobs::save(root,j)?;Ok(true)
}
