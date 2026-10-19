use super::err;
use crate::contracts::Result;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashSet};
pub fn assemble(p: &Value, test: &str, selected: &[String], version: u64) -> Result<Value> {
    if selected.is_empty() {
        return Err(err("请选择需要发布的题目单元"));
    }
    let mut sections: BTreeMap<String, Value> = BTreeMap::new();
    let mut asset_ids = HashSet::new();
    let mut ids = HashSet::new();
    for uid in selected {
        if !ids.insert(uid) {
            return Err(err("题组重复"));
        }
        let u = p["units"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|u| u["id"] == *uid)
            .ok_or_else(|| err("题组不存在"))?;
        if u["test"] != test
            || u["role"] != "questions"
            || u["state"] != "complete"
            || u["confirmed"] != true
        {
            return Err(err("只能发布同一个 Test 下已确认范围并处理完成的题目单元"));
        }
        let sec = u["section"].as_str().ok_or_else(|| err("题组缺少科目"))?;
        super::question_check::validate(&u["result"]["section"])?;
        let target=sections.entry(sec.into()).or_insert_with(||json!({"section":sec,"title":format!("{test} {sec}"),"durationMs":match sec{"listening"=>1800000,"reading"|"writing"=>3600000,_=>840000},"timingSource":"Academic section timing; verify imported scope","questions":[],"paragraphs":[]}));
        for para in u["result"]["section"]["paragraphs"]
            .as_array()
            .into_iter()
            .flatten()
        {
            target["paragraphs"]
                .as_array_mut()
                .unwrap()
                .push(para.clone());
        }
        for source_q in u["result"]["section"]["questions"]
            .as_array()
            .filter(|a| !a.is_empty())
            .ok_or_else(|| err("题组没有题目"))?
        {
            let mut q = source_q.clone();
            if q["conflicts"].as_array().is_some_and(|a| !a.is_empty()) {
                return Err(err(format!("第 {} 题存在答案冲突，请先确认", q["number"])));
            }
            if !["writing_task", "speaking_prompt"].contains(&q["kind"].as_str().unwrap_or("")) {
                if q["answer"]["accepted"]
                    .as_array()
                    .is_none_or(|a| a.is_empty())
                    || q["answerSource"]["quote"].as_str().is_none()
                {
                    return Err(err(format!(
                        "第 {} 题缺少有来源的答案；可留在资料库补传答案后再发布",
                        q["number"]
                    )));
                }
                if !q["evidence"].is_array() {
                    q["evidence"] = json!([])
                }
                let quote = q["answerSource"]["quote"].as_str().unwrap().to_string();
                q["evidence"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!(format!("ANSWER_SOURCE: {quote}")));
            }
            for key in ["answerConfirmed", "conflicts", "answerSource"] {
                q.as_object_mut().unwrap().remove(key);
            }
            if let Some(s) = q["imageAssetId"].as_str() {
                asset_ids.insert(s.to_string());
            }
            for s in q["imageAssetIds"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
            {
                asset_ids.insert(s.into());
            }
            if target["questions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|old| old["number"] == q["number"])
            {
                return Err(err(format!(
                    "{sec} 第 {} 题重复，请检查题组边界",
                    q["number"]
                )));
            }
            target["questions"].as_array_mut().unwrap().push(q);
        }
    }
    for (sec, s) in sections.iter_mut() {
        super::question_check::validate(s)?;
        s["questions"]
            .as_array_mut()
            .unwrap()
            .sort_by_key(|q| q["number"].as_u64().unwrap_or(0));
        if sec == "listening" {
            let required:Vec<_>=s["questions"].as_array().unwrap().iter().filter_map(|q|q["part"].as_u64()).collect();
            let docs=p["documents"].as_array().unwrap();
            let playlist=super::audio_assets::playlist(docs,test,&required)?;
            let mut parts=serde_json::Map::new();let mut list=vec![];let mut scripts=vec![];
            for track in &playlist {
                let id=track["assetId"].as_str().unwrap();
                let doc=docs.iter().find(|d|d["id"]==id).unwrap();
                if doc["audioVerified"]!=true{return Err(err(format!("音频 {} 尚未通过解码检查，请到听力音频中检查",doc["name"])));}
                list.push(json!(id));asset_ids.insert(id.to_string());
                if track["part"].is_number(){parts.insert(id.into(),track["part"].clone());}
                if let Some(turns)=doc["transcript"].as_array(){scripts.push(json!({"part":doc["binding"]["part"],"origin":doc["origin"],"turns":turns}));}
            }
            s["audioPlaylist"]=json!(playlist);s["audioScripts"]=json!(scripts);
            s["audioAssetIds"]=json!(list);s["audioParts"]=json!(parts);
            let unique:HashSet<_>=required.iter().copied().collect();
            if unique.len()<4 {let duration:u64=playlist.iter().map(|t|t["endMs"].as_u64().unwrap_or_else(||t["durationMs"].as_u64().unwrap_or(0)).saturating_sub(t["startMs"].as_u64().unwrap_or(0))).sum();if duration>0{s["durationMs"]=json!(duration.max(1000));s["timingSource"]=json!("所选音轨实测时长（单项/Part练习；不代表完整机考计时）");}}
        }
    }
    let mut assets = vec![];
    for id in asset_ids {
        let asset=p["documents"].as_array().into_iter().flatten().find_map(|d|{
            if d["id"]==id&&d["state"]=="media"{Some(json!({"id":id,"path":d["path"],"sha256":d["sha256"],"mediaType":match d["extension"].as_str().unwrap_or(""){"mp3"=>"audio/mpeg","ogg"=>"audio/ogg",_=>"audio/wav"}}))}
            else {d["assets"].as_array().into_iter().flatten().find(|a|a["id"]==id).cloned()}
        }).ok_or_else(||err(format!("缺少图片/音频 {id}")))?;
        assets.push(asset);
    }
    Ok(
        json!({"schemaVersion":1,"id":format!("source-{}-{}",p["id"].as_str().unwrap(),&crate::papers::hash(test.as_bytes())[..12]),"version":version,"title":format!("{} · {test}",p["title"].as_str().unwrap_or("资料")),"testType":"academic","status":"ready","provenance":{"kind":"imported","source":format!("source-library/{}, {test}; reviewed source-linked import",p["id"].as_str().unwrap())},"sections":sections.into_values().collect::<Vec<_>>(),"assets":assets}),
    )
}
