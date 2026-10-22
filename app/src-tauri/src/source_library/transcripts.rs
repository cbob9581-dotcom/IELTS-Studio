//! Source text is immutable; spoken drafts and silence events are explicit edits.
use super::{err,id};
use crate::contracts::Result;
use serde_json::{json,Value};
use regex::Regex;

fn clean(s:&str)->String {
 let s=s.replace("</tr>","\n").replace("</td>"," ").replace("<br>","\n").replace("<br/>","\n");
 let s=Regex::new(r"<[^>]*>").unwrap().replace_all(&s,"");
 let s=super::matching::decode_entities(&s);
 Regex::new(r"\bQ\d{1,2}(?:/\d{1,2})?\b").unwrap().replace_all(&s,"").into_owned()
}
pub fn from_blocks(blocks:&[Value],test:&str,part:u64)->Result<Value>{
 let mut lines=vec![];
 for b in blocks {
  let s=b["text"].as_str().unwrap_or("").trim();let upper=s.to_uppercase();
  if s.is_empty()||s.chars().all(|c|c.is_ascii_digit())||upper=="AUDIOSCRIPTS"||upper=="AUDIO SCRIPTS"||Regex::new(r"^(TEST|PART)\s+\d+$").unwrap().is_match(&upper){continue;}
  lines.push(clean(s));
 }
 let mut d=parse(&lines.join("\n"),"book",test,part)?;
 d["rawText"]=json!(blocks.iter().map(|b|b["text"].as_str().unwrap_or("")).collect::<Vec<_>>().join("\n"));
 d["sourceRefs"]=json!(blocks.iter().map(|b|json!({"blockId":b["id"],"page":b["page"]})).collect::<Vec<_>>());
 Ok(d)
}
pub fn parse(text:&str,origin:&str,test:&str,part:u64)->Result<Value>{
 if text.trim().is_empty()||text.len()>200000||test.trim().is_empty()||test.len()>100||!(1..=4).contains(&part){return Err(err("请填写听力文本、Test 和 Part 1–4；文本上限200KB"));}
 let spoken=clean(text);
 let labels=Regex::new(r"(?m)(?:^|\s)((?:[Ss][Pp][Ee][Aa][Kk][Ee][Rr] [A-Za-z0-9]+)|[A-Za-z][A-Za-z'\-]{0,24}):\s*").unwrap();
 let candidates:Vec<_>=labels.captures_iter(&spoken).collect();
 let mut known=std::collections::HashSet::new();
 for c in &candidates {let label=c.get(1).unwrap();let prefix=&spoken[..label.start()];let at_line_start=prefix.rsplit('\n').next().unwrap_or("").trim().is_empty();let after_sentence=prefix.trim_end().ends_with(['.','!','?']);let upper=c[1].chars().filter(|c|c.is_ascii_alphabetic()).all(|c|c.is_ascii_uppercase());if at_line_start||upper||after_sentence {known.insert(c[1].to_uppercase());}}
 let matches:Vec<_>=candidates.into_iter().filter(|c|known.contains(&c[1].to_uppercase())).collect();
 let mut raw:Vec<(String,String)>=vec![];
 if matches.is_empty(){for p in spoken.split("\n\n").filter(|s|!s.trim().is_empty()){raw.push(("NARRATOR".into(),p.into()));}}
 else {
  let first=matches[0].get(0).unwrap().start();
  if !spoken[..first].trim().is_empty(){raw.push(("NARRATOR".into(),spoken[..first].trim().into()));}
  for (i,c) in matches.iter().enumerate(){
   let m=c.get(0).unwrap();let start=m.end();let end=matches.get(i+1).map(|c|c.get(1).unwrap().start()).unwrap_or(spoken.len());
   raw.push((c[1].trim().to_uppercase(),spoken[start..end].trim().into()));
  }
 }
 let mut turns=vec![];let mut events=vec![];let mut previous=String::new();
 for (speaker,body) in raw {
  let body=body.split_whitespace().collect::<Vec<_>>().join(" ");if body.is_empty(){continue;}
  for (i,chunk) in chunks(&body,550).iter().enumerate(){
   if !turns.is_empty() && i==0 {events.push(json!({"type":"silence","durationMs":if previous==speaker{600}else{300}}));}
   let tid=format!("t{}",turns.len()+1);turns.push(json!({"id":tid,"speaker":speaker,"text":chunk}));events.push(json!({"type":"speech","turnId":tid}));
  }
  previous=speaker;
 }
 if turns.is_empty(){return Err(err("清理标题和题号标注后没有可朗读正文，请核对原文"));}
 Ok(json!({"id":id(),"revision":0,"origin":origin,"test":test.trim(),"part":part,"rawText":text,"turns":turns,"events":events,"voices":{},"sourceRefs":[],"state":"draft"}))
}
pub(crate) fn chunks(text:&str,max:usize)->Vec<String>{
 let mut out=vec![];let mut rest=text.trim();
 while rest.chars().count()>max {
  let limit=rest.char_indices().nth(max).map(|(i,_)|i).unwrap_or(rest.len());
  let head=&rest[..limit];let cut=head.char_indices().filter(|(_,c)|".!?".contains(*c)).map(|(i,c)|i+c.len_utf8()).filter(|i|*i>limit/3).last().or_else(||head.rfind(' ')).unwrap_or(limit);
  out.push(rest[..cut].trim().to_string());rest=rest[cut..].trim_start();
 }
 if !rest.is_empty(){out.push(rest.into());}out
}
pub fn validate(d:&Value)->Result<()> {
 let turns=d["turns"].as_array().filter(|a|!a.is_empty()&&a.len()<=2000).ok_or_else(||err("听力稿需要1–2000段正文"))?;
 let mut ids=std::collections::HashSet::new();
 for t in turns {let tid=t["id"].as_str().filter(|s|crate::contracts::valid_id(s)).ok_or_else(||err("正文段标识无效"))?;if !ids.insert(tid){return Err(err("正文段标识重复"));}if t["speaker"].as_str().is_none_or(|s|s.trim().is_empty()||s.len()>100)||t["text"].as_str().is_none_or(|s|s.trim().is_empty()||s.chars().count()>600){return Err(err("每段需要角色和正文，正文不超过600字符，请按句拆段"));}}
 let events=d["events"].as_array().filter(|a|!a.is_empty()&&a.len()<=5000).ok_or_else(||err("朗读时间线为空或过长"))?;
 let mut used=std::collections::HashSet::new();let mut silence=0;
 for e in events {match e["type"].as_str(){Some("speech")=>{let t=e["turnId"].as_str().unwrap_or("");if !ids.contains(t)||!used.insert(t){return Err(err("朗读时间线存在缺失或重复正文"));}},Some("silence")=>{let n=e["durationMs"].as_u64().filter(|n|*n<=120000).ok_or_else(||err("单次停顿须在0–120秒之间"))?;silence+=n;},_=>return Err(err("时间线事件无效"))}}
 if used.len()!=ids.len()||silence>1800000{return Err(err("时间线遗漏正文，或总停顿超过30分钟"));}Ok(())
}
