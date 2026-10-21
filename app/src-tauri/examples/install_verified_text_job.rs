//! Install an already approved local verification job. No cloud requests.
use ielts_core::{cloud::jobs,contracts::{StorageRoots,now_ms},papers,storage::Store};
use serde_json::{json,Value};
use std::{fs,path::{Path,PathBuf}};
fn copy_tree(base:&Path,source:&Path,target:&Path)->std::io::Result<()> {
 fs::create_dir_all(target)?;
 for e in fs::read_dir(source)?{let e=e?;let path=e.path();if !path.canonicalize()?.starts_with(base){return Err(std::io::Error::other("source link escapes task"));}let name=e.file_name();if ["job.json","status.json"].iter().any(|n|name==*n){continue;}if path.is_dir(){copy_tree(base,&path,&target.join(name))?;}else{fs::copy(path,target.join(name))?;}}
 Ok(())
}
fn main()->Result<(),Box<dyn std::error::Error>>{
 if std::env::var("IELTS_INSTALL_VERIFIED_TEXT").as_deref()!=Ok("1"){return Err("Explicit local install flag required".into());}
 let source=PathBuf::from(std::env::var("IELTS_VERIFIED_SOURCE_ROOT")?);let report:Value=serde_json::from_slice(&fs::read(source.join("last-text-verification.json"))?)?;
 let id=report["id"].as_str().ok_or("missing job")?;let mut j=jobs::load(&source,id)?;
 if !jobs::content_approved(&j)||j["audioPending"]==true||j["kind"]!="generation"{return Err("Job is not a complete approved text paper".into());}
 let roots=StorageRoots{assets:std::env::var("IELTS_VERIFIED_INSTALL_ASSETS")?,study:std::env::var("IELTS_VERIFIED_INSTALL_STUDY")?};let store=Store::reopen(roots)?;
 let source_dir=jobs::dir(&source,id)?.canonicalize()?;let target=Path::new(&store.roots.assets).join("cloud-jobs/generation").join(id);if target.exists(){return Err("Target task already exists; refusing overwrite".into());}
 let mut paper=j["paper"].clone();paper["status"]=json!("ready");papers::validate(&paper)?;
 copy_tree(&source_dir,&source_dir,&target)?;let installed=papers::install(&store,&paper,&source_dir)?;
 j["paper"]=installed;j["state"]=json!("complete");j["publishedAt"]=json!(now_ms());jobs::save(Path::new(&store.roots.assets),&j)?;
 println!("{}",json!({"jobId":id,"paperId":j["paper"]["id"],"title":j["paper"]["title"],"state":j["state"],"sections":j["paper"]["sections"].as_array().unwrap().iter().map(|s|json!({"section":s["section"],"questions":s["questions"].as_array().unwrap().len()})).collect::<Vec<_>>(),"cloudRequests":0,"ttsRequests":0}));Ok(())
}
