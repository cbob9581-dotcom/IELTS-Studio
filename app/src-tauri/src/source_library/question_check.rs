use super::err;
use crate::contracts::Result;
use serde_json::Value;
pub fn validate(section:&Value)->Result<()> {
 let mut covered=std::collections::HashSet::new();
 for q in section["questions"].as_array().into_iter().flatten(){
  let n=q["number"].as_u64().unwrap_or(0);let kind=q["kind"].as_str().unwrap_or("");
  if ["single_choice","multiple_choice","matching","heading_match"].contains(&kind)&&q["options"].as_array().is_none_or(|a|a.is_empty()){return Err(err(format!("第 {n} 题缺少 options；请保留原书选项，字母匹配至少提供原有字母列表")))}
  if kind=="matching"&&q["slots"].as_array().is_none_or(|a|a.is_empty()){return Err(err(format!("第 {n} 题缺少 slots；每个编号应提供一个可答题的匹配槽位")))}
  if ["listening","reading"].contains(&section["section"].as_str().unwrap_or("")){
   let points=q["points"].as_u64().filter(|n|(1..=3).contains(n)).ok_or_else(||err(format!("第 {n} 题分值无效")))?;
   let prompt=q["prompt"].as_str().unwrap_or("").to_uppercase();
   let expected=if prompt.contains("WHICH TWO")||prompt.contains("CHOOSE TWO"){2}else if prompt.contains("WHICH THREE")||prompt.contains("CHOOSE THREE"){3}else{points};
   if (points>1&&kind!="multiple_choice")||expected!=points{return Err(err(format!("第 {n} 题选 {expected} 项，应合为一个 multiple_choice、points={expected}，覆盖连续题号；不要拆成重复题目")))}
   for number in n..n+points {if !covered.insert(number){return Err(err(format!("题号 {number} 重复或被多选组合覆盖；保留一个组合题即可")))}}
  }
 }
 Ok(())
}
