use super::{err,dir};
use crate::contracts::Result;
use serde_json::{json,Value};
use std::{fs,path::Path};
pub fn store(root:&Path,p:&mut Value,did:&str,page:u64,bytes:&[u8])->Result<String>{
 let index=p["documents"].as_array().unwrap().iter().position(|d|d["id"]==did).ok_or_else(||err("原件不存在"))?;
 let d=&p["documents"][index];if d["extension"]!="pdf"||page==0||page>d["pageCount"].as_u64().unwrap_or(0){return Err(err("原 PDF 页码无效"))}
 if bytes.len()>12*1024*1024{return Err(err("原页题图超过12MB"))}
 let mut reader=png::Decoder::new(std::io::Cursor::new(bytes)).read_info().map_err(|_|err("原页题图不是有效 PNG"))?;
 if reader.info().width as u64*reader.info().height as u64>8_000_000{return Err(err("原页题图像素过大"))}
 let mut decoded=vec![0;reader.output_buffer_size()];reader.next_frame(&mut decoded).map_err(|_|err("原页 PNG 数据损坏"))?;
 let id=format!("{did}-page-{page}");let path=format!("media/{id}.png");let file=dir(root,p["id"].as_str().unwrap())?.join(&path);
 fs::create_dir_all(file.parent().unwrap())?;fs::write(&file,bytes)?;
 let d=&mut p["documents"][index];if !d["pageImages"].is_object(){d["pageImages"]=json!({});}d["pageImages"][page.to_string()]=json!(id);
 if !d["assets"].is_array(){d["assets"]=json!([])}d["assets"].as_array_mut().unwrap().retain(|a|a["id"]!=id);
 d["assets"].as_array_mut().unwrap().push(json!({"id":id,"path":path,"sha256":crate::papers::hash(bytes),"mediaType":"image/png"}));Ok(id)
}
