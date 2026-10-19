use super::err;
use crate::contracts::Result;
use serde_json::{json, Value};
pub fn candidates(questions: &Value, e: &Value) -> Vec<String> {
    if e["test"].as_str().is_none_or(|s| s.is_empty()) || e["section"].as_str().is_none() {
        return vec![];
    }
    questions
        .as_array()
        .into_iter()
        .flatten()
        .filter(|q| {
            q["test"] == e["test"]
                && q["section"] == e["section"]
                && e["numbers"]
                    .as_array()
                    .is_some_and(|a| a.contains(&q["number"]))
                && e.get("part").is_none_or(|p| p.is_null() || q["part"] == *p)
        })
        .filter_map(|q| q["id"].as_str().map(String::from))
        .collect()
}
pub fn verify_entry(e: &Value, blocks: &Value) -> Result<()> {
    let ids = e["blockIds"]
        .as_array()
        .filter(|a| !a.is_empty())
        .ok_or_else(|| err("提取条目缺少原文位置"))?;
    let mut text = String::new();
    for id in ids {
        let b = blocks
            .as_array()
            .into_iter()
            .flatten()
            .find(|b| b["id"] == *id)
            .ok_or_else(|| err("提取条目引用了不存在的版面块"))?;
        text.push_str(b["text"].as_str().unwrap_or(""));
        text.push('\n');
    }
    let quote = e["quote"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| err("提取条目缺少原文引文"))?;
    if !text.contains(quote) && !decode_entities(&text).contains(&decode_entities(quote)) {
        return Err(err("提取引文不在指定原文中，已保留响应等待核对"));
    }
    Ok(())
}
pub(crate) fn decode_entities(text:&str)->String{
    let mut out=String::with_capacity(text.len());let mut rest=text;
    while let Some(at)=rest.find('&'){
        out.push_str(&rest[..at]);rest=&rest[at..];
        let end=rest.find(';').filter(|n|*n<=12);
        let decoded=end.and_then(|end|{let entity=&rest[1..end];match entity {"amp"=>Some('&'),"lt"=>Some('<'),"gt"=>Some('>'),"quot"=>Some('"'),"apos"=>Some('\''),"nbsp"=>Some('\u{a0}'),_=>entity.strip_prefix("#x").or_else(||entity.strip_prefix("#X")).and_then(|s|u32::from_str_radix(s,16).ok()).or_else(||entity.strip_prefix('#').and_then(|s|s.parse().ok())).and_then(char::from_u32)}});
        if let (Some(end),Some(c))=(end,decoded){out.push(c);rest=&rest[end+1..];}else{out.push('&');rest=&rest[1..];}
    }
    out.push_str(rest);out
}
pub fn attach(q: &mut Value, e: &Value, force: bool) -> Result<()> {
    if e["kind"] == "answers" {
        if e["answer"]["accepted"]
            .as_array()
            .is_none_or(|a| a.is_empty())
        {
            return Err(err("答案条目没有允许的答案"));
        }
        let mut answer=e["answer"].clone();
        if ["single_choice","multiple_choice","matching","heading_match"].contains(&q["kind"].as_str().unwrap_or("")) {
            fn option(v:&Value,options:&Value)->Value {
                let Some(s)=v.as_str() else{return v.clone()};
                let Some(options)=options.as_array().filter(|a|!a.is_empty()) else{return v.clone()};
                if options.iter().any(|o|o==s){return v.clone()}
                let label=s.to_lowercase();
                if let Some(o)=options.iter().find(|o|o.as_str().is_some_and(|text|[".",")",":"," "].iter().any(|sep|text.to_lowercase().starts_with(&format!("{label}{sep}"))))){return o.clone()}
                if s.len()==1&&s.as_bytes()[0].is_ascii_uppercase(){if let Some(o)=options.get((s.as_bytes()[0]-b'A') as usize){return o.clone()}}
                v.clone()
            }
            for accepted in answer["accepted"].as_array_mut().into_iter().flatten(){
                if let Some(a)=accepted.as_array_mut(){for v in a {*v=option(v,&q["options"]);}}
                else if let Some(a)=accepted.as_object_mut(){for v in a.values_mut(){*v=option(v,&q["options"]);}}
                else if accepted.is_string(){*accepted=option(accepted,&q["options"]);}
                if q["kind"]=="matching"&&accepted.is_string(){if let Some(slot)=q["slots"].as_array().filter(|a|a.len()==1).and_then(|a|a[0].as_str()){let mut value=serde_json::Map::new();value.insert(slot.into(),accepted.clone());*accepted=Value::Object(value);}}
            }
        }
        if q.get("answer")
            .is_some_and(|a| !a.is_null() && *a != answer)
            && !force
        {
            if !q["conflicts"].is_array() {
                q["conflicts"] = json!([])
            }
            if !q["conflicts"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v["id"] == e["id"])
            {
                q["conflicts"].as_array_mut().unwrap().push(e.clone());
            }
            return Ok(());
        }
        q["answer"] = answer;
        q["answerConfirmed"] = json!(true);
        q["answerSource"] = e.clone();
        if force {
            q["conflicts"] = json!([])
        }
    } else {
        if !q["reviewMaterials"].is_array() {
            q["reviewMaterials"] = json!([])
        }
        let a = q["reviewMaterials"].as_array_mut().unwrap();
        if !a.iter().any(|v| v["id"] == e["id"]) {
            a.push(e.clone())
        }
    }
    Ok(())
}
pub fn reconcile(p: &mut Value) -> Result<()> {
    // Rebuild derived attachments from surviving source entries; removing a source must invalidate its answer.
    for u in p["units"].as_array_mut().into_iter().flatten() {
        if u["role"]!="questions" {continue;}
        for q in u["result"]["section"]["questions"]
            .as_array_mut()
            .into_iter()
            .flatten()
        {
            if q.get("answerSource").is_some() {
                q.as_object_mut().unwrap().remove("answer");
            }
            for k in [
                "answerSource",
                "answerConfirmed",
                "conflicts",
                "reviewMaterials",
            ] {
                q.as_object_mut().unwrap().remove(k);
            }
        }
    }
    let mut questions = vec![];
    for u in p["units"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|u| u["role"] == "questions")
    {
        if u["role"]!="questions" {continue;}
        for q in u["result"]["section"]["questions"]
            .as_array()
            .into_iter()
            .flatten()
        {
            let mut v = q.clone();
            v["test"] = u["test"].clone();
            v["section"] = u["section"].clone();
            questions.push(v);
        }
    }
    let all = json!(questions);
    let mut entries = p["entries"].as_array().cloned().unwrap_or_default();
    for e in entries.iter_mut() {
        if e["state"] == "ignored" {
            continue;
        }
        let ids = candidates(&all, e);
        e["candidates"] = json!(ids);
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        let expected = e["numbers"].as_array().map(Vec::len).unwrap_or(0);
        if ids.is_empty() || ids.len() != expected || unique.len() != ids.len() {
            e["state"] = json!("unassigned");
            continue;
        }
        let mut conflict = false;
        for u in p["units"].as_array_mut().unwrap() {
            if u["role"]!="questions" {continue;}
            for q in u["result"]["section"]["questions"]
                .as_array_mut()
                .into_iter()
                .flatten()
            {
                if ids.iter().any(|id| q["id"] == *id) {
                    attach(q, e, false)?;
                    if q["conflicts"].as_array().is_some_and(|a| !a.is_empty()) {
                        conflict = true
                    }
                }
            }
        }
        e["state"] = json!(if conflict { "conflict" } else { "matched" });
    }
    p["entries"] = json!(entries);
    Ok(())
}
