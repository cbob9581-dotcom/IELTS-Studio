//! Explicit text-service verification. TTS access is denied even if a workflow regresses.
use ielts_core::{cloud::{self,generation_workflow::{self,Providers},jobs},configuration::{self,ProviderConfig},contracts::Result,credentials,papers,storage::atomic_json};
use serde_json::{json,Value};
use std::{fs,path::PathBuf};
struct TextOnly{config:PathBuf}
impl Providers for TextOnly {
 fn provider(&self,id:&str)->Result<(ProviderConfig,String)>{
  if id!="deepseek"{return Err(cloud::error("Verification forbids ALL TTS and other service access"));}
  let cfg=configuration::load(&self.config)?;let p=cfg.providers.get(id).ok_or_else(||cloud::error("Missing text provider"))?.clone();
  if !p.enabled{return Err(cloud::error("Text provider is disabled"));}Ok((p,credentials::read_for_service(id)?))
 }
}
#[tokio::main]
async fn main()->std::result::Result<(),Box<dyn std::error::Error>>{
 if std::env::var("IELTS_ALLOW_TEXT_CLOUD_TEST").as_deref()!=Ok("1"){return Err("Set IELTS_ALLOW_TEXT_CLOUD_TEST=1 to explicitly allow text requests; TTS remains forbidden".into());}
 let target=PathBuf::from(std::env::var("IELTS_TEXT_TEST_ROOT")?);let fixture=std::env::var("IELTS_TEXT_TEST_FIXTURE").ok().map(PathBuf::from);let services=TextOnly{config:std::env::var("IELTS_TEXT_TEST_CONFIG")?.into()};
 let request:Value=serde_json::from_str(&std::env::var("IELTS_TEXT_TEST_REQUEST")?)?;let mut j=jobs::create(&target,"generation",request)?;let dir=jobs::dir(&target,j["id"].as_str().unwrap())?;
 if let Some(source)=fixture {for f in fs::read_dir(source)?{let f=f?;let name=f.file_name().to_string_lossy().to_string();if f.path().is_file()&&name.starts_with("part-")&&name.ends_with(".json"){fs::copy(f.path(),dir.join(name))?;}}}
 let mut steps=0;
 while steps<32&&!matches!(j["state"].as_str(),Some("review_required"|"failed")){
  match generation_workflow::step(&services,&target,&mut j).await {Ok(())=>{},Err(e)=>{j["state"]=json!("failed");j["error"]=json!(e.message);}}
  jobs::save(&target,&j)?;steps+=1;println!("{}",json!({"step":steps,"state":j["state"],"stage":j["stage"],"operation":j["operation"]["label"],"ttsRequests":0}));
 }
 let report=json!({"id":j["id"],"state":j["state"],"error":j["error"],"contentApproved":jobs::content_approved(&j),"validation":j["validation"],"audioPending":j["audioPending"],"ttsRequests":0,"steps":steps});
 atomic_json(&target.join("last-text-verification.json"),&report)?;println!("{report}");
 if j["state"]!="review_required"||!jobs::content_approved(&j){return Err("Text verification did not reach approved draft; original inputs preserved".into());}
 let mut paper=j["paper"].clone();if j["audioPending"]!=true {paper["status"]=json!("ready");papers::validate(&paper)?;atomic_json(&target.join("approved-paper.json"),&paper)?;}
 Ok(())
}
