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
 if d["state"]=="failed" {
  let target=find_mut(p,"documents",did)?;
  for segment in target["parseSegments"].as_array_mut().unwrap(){if segment["state"]=="failed"{segment["state"]=json!("pending");segment["error"]=Value::Null;}}
  target["state"]=json!("parsing");target["error"]=Value::Null;return Ok(None)
 }
 let indices=pdf_batch::pending_indices(&d,crate::task_policy::TaskPolicy::load(r)?.ocr_concurrency)?;
 let mut created=false;
 for index in &indices {
  let segment=&d["parseSegments"][*index];
  if segment["parseJobId"].is_string(){continue;}
  let path=PathBuf::from(segment["path"].as_str().ok_or_else(||err("PDF 批次路径缺失"))?).canonicalize()?;
  if !path.starts_with(project.canonicalize()?){return Err(err("PDF 批次路径越界"))}
  let language=if d["roles"].as_array().into_iter().flatten().any(|v|matches!(v.as_str(),Some("mixed"|"explanations"|"essays"))){"ch"}else{"en"};
  let j=cloud_commands::cloud_create("import".into(),json!({"sourceOnly":true,"section":"all","repairOcr":false,"language":language}),Some(path.to_string_lossy().into()),state.clone())?;
  find_mut(p,"documents",did)?["parseSegments"][*index]["parseJobId"]=j["id"].clone();created=true;
 }
 // Persist every child ID before sending a provider request; this return is committed by source_step.
 if created{return Ok(None)}
 let mut results=vec![];
 if indices.len()==2 {
  let first=d["parseSegments"][indices[0]]["parseJobId"].as_str().unwrap().to_owned();let second=d["parseSegments"][indices[1]]["parseJobId"].as_str().unwrap().to_owned();
  let (a,b)=tokio::join!(cloud_commands::cloud_step(first,state.clone()),cloud_commands::cloud_step(second,state.clone()));results.push((indices[0],a));results.push((indices[1],b));
 }else if let Some(index)=indices.first(){results.push((*index,cloud_commands::cloud_step(d["parseSegments"][*index]["parseJobId"].as_str().unwrap().into(),state.clone()).await));}
 for (index,result) in results {
  let segment=&d["parseSegments"][index];let first=segment["firstPage"].as_u64().unwrap();let last=segment["lastPage"].as_u64().unwrap();
  let j=match result{Ok(j)=>j,Err(e)=>{pdf_batch::record_failure(find_mut(p,"documents",did)?,index,&e.message)?;continue;}};
  if ["failed","paused"].contains(&j["state"].as_str().unwrap_or("")){pdf_batch::record_failure(find_mut(p,"documents",did)?,index,j["error"].as_str().unwrap_or("云端解析未完成"))?;continue;}
  let t=find_mut(p,"documents",did)?;t["parseSegments"][index]["done"]=json!(if j["state"]=="complete"{last-first+1}else{j["operation"]["done"].as_u64().unwrap_or(0).min(last-first+1)});
  if j["state"]!="complete"{continue;}
  let mut media_scope=d.clone();media_scope["id"]=json!(format!("{did}-s{index}"));
  match import_blocks(r,p,&media_scope,&j){
   Ok((blocks,assets))=>{lib::save_blocks(r,p,&format!("{did}-s{index}"),&blocks)?;let t=find_mut(p,"documents",did)?;t["parseSegments"][index]["state"]=json!("complete");t["parseSegments"][index]["assets"]=json!(assets);t["parseSegments"][index]["blockCount"]=json!(blocks.len());},
   Err(e)=>pdf_batch::record_failure(find_mut(p,"documents",did)?,index,&e.message)?
  }
 }
 let current=find(p,"documents",did)?.clone();let segments=current["parseSegments"].as_array().unwrap();
 let done:u64=segments.iter().map(|s|if s["state"]=="complete"{s["lastPage"].as_u64().unwrap()-s["firstPage"].as_u64().unwrap()+1}else{s["done"].as_u64().unwrap_or(0)}).sum();
 let failures:Vec<_>=segments.iter().filter(|s|s["state"]=="failed").map(|s|format!("原书第 {}–{} 页：{}",s["firstPage"],s["lastPage"],s["error"].as_str().unwrap_or("未完成"))).collect();
 let t=find_mut(p,"documents",did)?;t["state"]=json!("parsing");t["operation"]=json!({"phase":"parsing","label":format!("OCR 最多两批并发 · {} 批完成，{} 批需处理",segments.iter().filter(|s|s["state"]=="complete").count(),failures.len()),"done":done,"total":d["pageCount"],"unit":"页"});
 if !pdf_batch::pending_indices(&current,2)?.is_empty(){return Ok(None)}
 if !failures.is_empty(){t["state"]=json!("failed");t["error"]=json!(format!("{}。已完成批次保留，继续只处理失败批次",failures.join("；")));return Ok(None)}
 let d=find(p,"documents",did)?;let mut batches=vec![];let mut assets=vec![];
 for (i,s) in d["parseSegments"].as_array().unwrap().iter().enumerate(){
  batches.push((s["firstPage"].as_u64().unwrap() as u32,lib::blocks(r,p,&format!("{did}-s{i}"))?));assets.extend(s["assets"].as_array().cloned().unwrap_or_default());
 }
 Ok(Some((pdf_batch::merge(batches)?,assets)))
}
