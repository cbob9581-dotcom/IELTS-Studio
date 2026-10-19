use super::*;
use crate::source_library::pdf_batch;
pub(super) async fn step(r:&Path,p:&mut Value,did:&str,state:&State<'_,AppState>)->Result<Option<(Vec<Value>,Vec<Value>)>>{
 let d=find(p,"documents",did)?.clone();let project=lib::dir(r,p["id"].as_str().unwrap())?;
 if !d["parseSegments"].is_array(){
  let source=lib::document_path(r,p,&d)?;let dest=project.join("segments").join(did);
  let prepared=tokio::task::spawn_blocking(move||pdf_batch::prepare(&source,&dest,pdf_batch::MAX_PAGES,pdf_batch::MAX_BYTES)).await.map_err(|_|err("PDF 分批任务中断，原件保留"))??;
  let mut segments=prepared["segments"].clone();
  if segments.as_array().unwrap().len()==1&&d["parseJobId"].is_string(){segments[0]["parseJobId"]=d["parseJobId"].clone();}
  let target=find_mut(p,"documents",did)?;target["parseSegments"]=segments;target["pageCount"]=prepared["pageCount"].clone();target["state"]=json!("parsing");
  target["operation"]=json!({"phase":"parsing","label":format!("原件共 {} 页，分为 {} 批处理",prepared["pageCount"],prepared["segments"].as_array().unwrap().len()),"done":0,"total":prepared["pageCount"],"unit":"页"});
  return Ok(None)
 }
 let segments=d["parseSegments"].as_array().unwrap();
 if let Some(index)=segments.iter().position(|s|s["state"]!="complete"){
  let segment=segments[index].clone();let first=segment["firstPage"].as_u64().unwrap();let last=segment["lastPage"].as_u64().unwrap();
  let path=PathBuf::from(segment["path"].as_str().ok_or_else(||err("PDF 批次路径缺失"))?).canonicalize()?;
  if !path.starts_with(project.canonicalize()?){return Err(err("PDF 批次路径越界"))}
  let language=if d["roles"].as_array().into_iter().flatten().any(|v|matches!(v.as_str(),Some("mixed"|"explanations"|"essays"))){"ch"}else{"en"};
  let j=if let Some(jid)=segment["parseJobId"].as_str(){cloud_commands::cloud_step(jid.into(),state.clone()).await?}else{
   let j=cloud_commands::cloud_create("import".into(),json!({"sourceOnly":true,"section":"all","repairOcr":false,"language":language}),Some(path.to_string_lossy().into()),state.clone())?;
   find_mut(p,"documents",did)?["parseSegments"][index]["parseJobId"]=j["id"].clone();j
  };
  let completed:u64=segments.iter().filter(|s|s["state"]=="complete").map(|s|s["lastPage"].as_u64().unwrap()-s["firstPage"].as_u64().unwrap()+1).sum();
  let done=if j["state"]=="complete"{last-first+1}else{j["operation"]["done"].as_u64().unwrap_or(0).min(last-first+1)};
  let t=find_mut(p,"documents",did)?;t["state"]=json!("parsing");t["operation"]=json!({"phase":"parsing","label":format!("OCR 第 {}/{} 批 · 原书第 {first}–{last} 页",index+1,segments.len()),"done":completed+done,"total":d["pageCount"],"unit":"页"});
  if j["state"]=="failed"||j["state"]=="paused"{return Err(err(format!("原书第 {first}–{last} 页解析未完成：{}；其他批次保留",j["error"].as_str().unwrap_or("请检查云端任务"))))}
  if j["state"]!="complete"{return Ok(None)}
  let mut media_scope=d.clone();media_scope["id"]=json!(format!("{did}-s{index}"));
  let (blocks,assets)=import_blocks(r,p,&media_scope,&j)?;
  lib::save_blocks(r,p,&format!("{did}-s{index}"),&blocks)?;
  let t=find_mut(p,"documents",did)?;t["parseSegments"][index]["state"]=json!("complete");t["parseSegments"][index]["assets"]=json!(assets);t["parseSegments"][index]["blockCount"]=json!(blocks.len());
  if t["parseSegments"].as_array().unwrap().iter().any(|s|s["state"]!="complete"){return Ok(None)}
 }
 let d=find(p,"documents",did)?;let mut batches=vec![];let mut assets=vec![];
 for (i,s) in d["parseSegments"].as_array().unwrap().iter().enumerate(){
  batches.push((s["firstPage"].as_u64().unwrap() as u32,lib::blocks(r,p,&format!("{did}-s{i}"))?));assets.extend(s["assets"].as_array().cloned().unwrap_or_default());
 }
 Ok(Some((pdf_batch::merge(batches)?,assets)))
}
