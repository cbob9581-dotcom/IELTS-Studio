use super::err;
use crate::{contracts::Result,storage::atomic_json};
use serde_json::{json,Value};
use std::{fs,path::Path,io::Read};
pub const MAX_PAGES:u32=180;
pub const MAX_BYTES:u64=40*1024*1024;
pub fn pending_indices(document:&Value,concurrency:usize)->Result<Vec<usize>>{let segments=document["parseSegments"].as_array().ok_or_else(||err("缺少 PDF 批次记录"))?;Ok(segments.iter().enumerate().filter(|(_,s)|!["complete","failed"].contains(&s["state"].as_str().unwrap_or("pending"))).take(concurrency.clamp(1,3)).map(|(i,_)|i).collect())}
pub fn record_failure(document:&mut Value,index:usize,message:&str)->Result<()>{let segment=document["parseSegments"].as_array_mut().and_then(|s|s.get_mut(index)).ok_or_else(||err("PDF 批次不存在"))?;segment["state"]=json!("failed");segment["error"]=json!(message);Ok(())}
pub fn ranges(pages:u32,bytes:u64,max_pages:u32,max_bytes:u64)->Result<Vec<(u32,u32)>>{
 if pages==0||max_pages==0||max_bytes==0{return Err(err("PDF 页数或分批配置无效"))}
 let count=pages.div_ceil(max_pages).max(bytes.div_ceil(max_bytes).min(pages as u64) as u32).max(1);
 let chunk=pages.div_ceil(count);let mut out=vec![];let mut first=1;
 while first<=pages {let last=(first+chunk-1).min(pages);out.push((first,last));first=last+1;}Ok(out)
}
fn hash_file(path:&Path)->Result<String>{
 use sha2::{Digest,Sha256};let mut f=fs::File::open(path)?;let mut digest=Sha256::new();let mut b=[0;65536];
 loop{let n=f.read(&mut b)?;if n==0{break}digest.update(&b[..n]);}Ok(format!("{:x}",digest.finalize()))
}
pub fn prepare(source:&Path,dest:&Path,max_pages:u32,max_bytes:u64)->Result<Value>{
 let digest=hash_file(source)?;let manifest=dest.join("manifest.json");
 if manifest.is_file(){let old:Value=serde_json::from_slice(&fs::read(&manifest)?)?;
  if old["sourceHash"]==digest&&old["maxPages"]==max_pages&&old["maxBytes"]==max_bytes&&old["segments"].as_array().is_some_and(|a|a.iter().all(|s|s["path"].as_str().is_some_and(|p|Path::new(p).is_file()))){return Ok(old)}
 }
 let doc=lopdf::Document::load(source).map_err(|e|err(format!("PDF 分批读取失败，请检查文件是否损坏：{e}")))?;
 if doc.is_encrypted(){return Err(err("PDF 已加密，请先提供可读取的解密副本"))}
 let pages=doc.get_pages().len() as u32;let plan=ranges(pages,fs::metadata(source)?.len(),max_pages,max_bytes)?;
 fs::create_dir_all(dest)?;let mut segments=vec![];
 fn write(doc:&lopdf::Document,source:&Path,dest:&Path,pages:u32,first:u32,last:u32,max_bytes:u64,out:&mut Vec<Value>)->Result<()> {
  if first==1&&last==pages&&fs::metadata(source)?.len()<=max_bytes {out.push(json!({"firstPage":first,"lastPage":last,"path":source.to_string_lossy(),"sha256":hash_file(source)?,"state":"pending"}));return Ok(())}
  let mut part=doc.clone();let remove:Vec<_>=(1..=pages).filter(|p|*p<first||*p>last).collect();part.delete_pages(&remove);
  if let Ok(catalog)=part.catalog_mut(){catalog.remove(b"Outlines");catalog.remove(b"Names");}
  part.prune_objects();part.renumber_objects();
  let target=dest.join(format!("pages-{first:04}-{last:04}.pdf"));let temp=dest.join(format!("pages-{first:04}-{last:04}.pending"));
  part.save(&temp).map_err(|e|err(format!("PDF 分批保存失败：{e}")))?;
  if fs::metadata(&temp)?.len()>max_bytes {
   // This file was created above under the fixed document segments directory.
   fs::remove_file(&temp)?;
   if first==last{return Err(err(format!("原书第 {first} 页资源过大，无法在当前单批上限内处理")))}
   let middle=(first+last)/2;write(doc,source,dest,pages,first,middle,max_bytes,out)?;write(doc,source,dest,pages,middle+1,last,max_bytes,out)?;return Ok(())
  }
  fs::rename(&temp,&target)?;
  out.push(json!({"firstPage":first,"lastPage":last,"path":target.to_string_lossy(),"sha256":hash_file(&target)?,"state":"pending"}));Ok(())
 }
 for (first,last) in plan{write(&doc,source,dest,pages,first,last,max_bytes,&mut segments)?;}
 let v=json!({"sourceHash":digest,"pageCount":pages,"maxPages":max_pages,"maxBytes":max_bytes,"segments":segments});atomic_json(&manifest,&v)?;Ok(v)
}
pub fn merge(batches:Vec<(u32,Vec<Value>)>)->Result<Vec<Value>>{
 let mut out=vec![];let mut last_start=0;
 for (first,blocks) in batches {
  if first==0||first<=last_start{return Err(err("PDF 批次顺序错误"))}last_start=first;
  for mut b in blocks {if let Some(page)=b["page"].as_u64(){b["page"]=json!(page+first as u64-1);}
   b["batchPage"]=json!(b["page"].as_u64().map(|p|p-first as u64+1));b["id"]=json!(format!("b{}",out.len()+1));out.push(b);
  }
 }Ok(out)
}
