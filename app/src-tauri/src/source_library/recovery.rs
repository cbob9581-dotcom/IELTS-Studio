//! Adopt a completed parse from the legacy single-paper flow without another OCR request.
use super::{self as lib,layout,outline,err};
use crate::{contracts::Result,papers,storage::atomic_json};
use serde_json::{json,Value};
use std::{fs,path::{Path,PathBuf}};
fn walk(base:&Path,path:&Path,out:&mut Vec<PathBuf>)->Result<()> {
 if !path.is_dir(){return Ok(());}
 for entry in fs::read_dir(path)? {let path=entry?.path();let resolved=path.canonicalize()?;if !resolved.starts_with(base){return Err(err("解析缓存链接越界"));}if path.is_dir(){walk(base,&path,out)?;}else{out.push(path);}}
 Ok(())
}
pub fn adopt_from(root:&Path,source:&Path,job:&Value)->Result<Value> {
 let jid=job["id"].as_str().filter(|s|crate::contracts::valid_id(s)).ok_or_else(||err("导入任务编号无效"))?;
 if job["kind"]!="import"||!source.join("original.md").is_file(){return Err(err("此任务还没有可复用的解析结果，请先完成文件解析"));}
 for item in lib::list(root)? {let p=lib::load(root,item["id"].as_str().unwrap())?;if p["recoveredJobId"]==jid{return Ok(p);}}
 let ext=job["request"]["extension"].as_str().filter(|s|["pdf","txt","md","png","jpg","jpeg","webp","doc","docx","ppt","pptx"].contains(s)).ok_or_else(||err("原件格式不支持恢复"))?;
 let original=source.join(format!("source.{ext}"));let bytes=fs::read(&original)?;
 if job["request"]["sha256"].as_str().is_some_and(|h|h!=papers::hash(&bytes)){return Err(err("原件哈希与导入任务不一致，未更改原任务"));}
 let base=source.canonicalize()?;let mut files=vec![];walk(&base,&source.join("extracted"),&mut files)?;files.sort();
 let structure=files.iter().find(|f|f.file_name().and_then(|s|s.to_str()).is_some_and(|s|s.ends_with("_content_list.json")||s=="content_list.json")).or_else(||files.iter().find(|f|f.file_name().is_some_and(|s|s=="structured_content.json")));
 let mut blocks=if let Some(path)=structure {layout::normalize(&serde_json::from_slice(&fs::read(path)?)?)?}else{layout::plain(&fs::read_to_string(source.join("original.md"))?)};
 if blocks.is_empty(){return Err(err("已保存的解析内容为空，未重新发送 OCR"));}
 let mut p=lib::create(root,job["request"]["sourceName"].as_str().unwrap_or("已解析资料"))?;
 let did=lib::add_document(root,&mut p,&original,json!(["mixed"]))?;let folder=lib::dir(root,p["id"].as_str().unwrap())?;
 let mut assets=vec![];
 for file in files.iter().filter(|f|f.extension().and_then(|s|s.to_str()).is_some_and(|s|["png","jpg","jpeg","webp"].contains(&s))){
  let rel=file.strip_prefix(source).map_err(err)?.to_string_lossy().replace('\\',"/");let bytes=fs::read(file)?;let hash=papers::hash(&bytes);let ext=file.extension().unwrap().to_str().unwrap();let aid=format!("recovered-{}",&hash[..24]);let path=format!("media/{aid}.{ext}");fs::create_dir_all(folder.join("media"))?;fs::write(folder.join(&path),&bytes)?;
  for block in &mut blocks {let ip=block["imagePath"].as_str().unwrap_or("").replace('\\',"/");if !ip.is_empty()&&(rel==ip||rel.ends_with(&format!("/{ip}"))){block["imageAssetId"]=json!(aid);}}
  if !assets.iter().any(|a:&Value|a["id"]==aid){assets.push(json!({"id":aid,"path":path,"sha256":hash,"mediaType":if ext=="png"{"image/png"}else if ext=="webp"{"image/webp"}else{"image/jpeg"}}));}
 }
 lib::save_blocks(root,&p,&did,&blocks)?;
 let specs=outline::derive(&blocks,&json!(["mixed"]));let mut units=vec![];
 for spec in &specs {let mut u=lib::make_unit(&p,&did,spec,&blocks)?;u["confirmed"]=json!(false);u["suggestedBy"]=json!("printed_headings");units.push(u);}
 let doc=p["documents"].as_array_mut().unwrap().iter_mut().find(|d|d["id"]==did).unwrap();doc["name"]=job["request"]["sourceName"].clone();doc["state"]=json!(if units.is_empty(){"scanning"}else{"ready"});doc["blockCount"]=json!(blocks.len());doc["pageCount"]=json!(blocks.iter().filter_map(|b|b["page"].as_u64()).max().unwrap_or(0));doc["assets"]=json!(assets);doc["parseJobId"]=json!(jid);doc["indexMethod"]=json!("reused_parse");doc["scanCursor"]=json!(if units.is_empty(){0}else{blocks.len()});
 p["units"]=json!(units);p["recoveredJobId"]=json!(jid);p["recoveryNote"]=json!("已复用原件、解析版面与图片，未重新 OCR；旧失败模型草稿未作为答案来源。请按 Test 核对题组与答案页。");
 atomic_json(&folder.join("recovery.json"),&json!({"jobId":jid,"sha256":job["request"]["sha256"],"reusedParse":true}))?;lib::save(root,&mut p,0)?;Ok(p)
}
