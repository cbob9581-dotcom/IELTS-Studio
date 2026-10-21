//! Resolve immutable source positions locally; model quotations never become source text.
use super::{err, matching::decode_entities};
use crate::contracts::Result;
use serde_json::{json, Value};
fn lines(block: &Value) -> Vec<Value> {
    block["text"].as_str().unwrap_or("").lines().enumerate().filter(|(_,s)|!s.trim().is_empty()).map(|(i,s)|json!({"id":format!("{}-l{}",block["id"].as_str().unwrap_or(""),i+1),"text":s})).collect()
}
pub fn for_model(blocks: &Value) -> Result<Value> {
    let blocks = blocks.as_array().ok_or_else(|| err("缺少原资料版面块"))?;
    Ok(json!(blocks
        .iter()
        .map(|b| {
            let mut b = b.clone();
            b["sourceLines"] = json!(lines(&b));
            b.as_object_mut().unwrap().remove("text");
            b
        })
        .collect::<Vec<_>>()))
}
fn normalized_span(text: &str, quote: &str) -> Option<String> {
    let source: Vec<char> = text.chars().collect();
    let mut normalized = Vec::new();
    let mut positions: Vec<(usize, usize)> = vec![];
    for (i, c) in source.iter().enumerate() {
        if c.is_whitespace() {
            if normalized.last() == Some(&' ') {
                positions.last_mut()?.1 = i + 1;
            } else {
                normalized.push(' ');
                positions.push((i, i + 1));
            }
        } else {
            normalized.push(*c);
            positions.push((i, i + 1));
        }
    }
    let needle: Vec<char> = quote
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .collect();
    if needle.is_empty() {
        return None;
    }
    let at = normalized.windows(needle.len()).position(|w| w == needle)?;
    Some(
        source[positions[at].0..positions[at + needle.len() - 1].1]
            .iter()
            .collect(),
    )
}
pub fn resolve(entry: &Value, blocks: &Value) -> Result<Value> {
    // Some models put an otherwise valid line position in the legacy blockIds key.
    // Resolve the actual position; never accept a guessed quote or strip an arbitrary suffix.
    if entry["sourceLineIds"].is_null() {
        if let Some(ids) = entry["blockIds"]
            .as_array()
            .filter(|a| !a.is_empty() && a.len() <= 100)
        {
            if ids.iter().all(|id| {
                !blocks
                    .as_array()
                    .into_iter()
                    .flatten()
                    .any(|b| b["id"] == *id)
                    && blocks
                        .as_array()
                        .into_iter()
                        .flatten()
                        .any(|b| lines(b).iter().any(|l| l["id"] == *id))
            }) {
                let mut fixed = entry.clone();
                fixed["sourceLineIds"] = json!(ids);
                return resolve(&fixed, blocks);
            }
        }
    }
    let blocks = blocks.as_array().ok_or_else(|| err("缺少原资料版面块"))?;
    let mut out = entry.clone();
    if let Some(ids) = entry["sourceLineIds"].as_array() {
        if ids.is_empty() || ids.len() > 100 {
            return Err(err("来源行范围无效"));
        }
        let mut quotes = vec![];
        let mut bids = vec![];
        for id in ids {
            let mut found = None;
            for b in blocks {
                if let Some(line) = lines(b).into_iter().find(|line| line["id"] == *id) {
                    found = Some((b, line));
                    break;
                }
            }
            let Some((b, line)) = found else {
                return Err(err("引用的来源行不在当前题组中"));
            };
            if !bids.contains(&b["id"]) {
                bids.push(b["id"].clone());
            }
            quotes.push(line["text"].as_str().unwrap().to_string());
        }
        out["blockIds"] = json!(bids);
        out["quote"] = json!(quotes.join("\n"));
        out["evidenceKind"] = json!("source_lines");
        out["sourceVerified"] = json!(true);
        return Ok(out);
    }
    if let Some(aid) = entry["sourceImageId"].as_str() {
        let selected: Vec<_> = blocks.iter().filter(|b| b["imageAssetId"] == aid).collect();
        let text = entry["transcription"]
            .as_str()
            .filter(|s| !s.trim().is_empty() && s.len() <= 20000)
            .ok_or_else(|| err("图片条目缺少可核对的转录文字"))?;
        if selected.is_empty() {
            return Err(err("图片来源不在当前题组中"));
        }
        out["blockIds"] = json!(selected.iter().map(|b| b["id"].clone()).collect::<Vec<_>>());
        out["quote"] = json!(text);
        out["evidenceKind"] = json!("visual_transcription");
        out["sourceVerified"] = json!(false);
        return Ok(out);
    }
    let ids = entry["blockIds"]
        .as_array()
        .filter(|a| !a.is_empty() && a.len() <= 100)
        .ok_or_else(|| err("提取条目缺少原文位置"))?;
    let mut text = String::new();
    for id in ids {
        let block = blocks
            .iter()
            .find(|b| b["id"] == *id)
            .ok_or_else(|| err("提取条目引用了不存在的版面块"))?;
        text.push_str(block["text"].as_str().unwrap_or(""));
        text.push('\n');
    }
    let quote = entry["quote"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| err("提取条目缺少原文引文"))?;
    if text.contains(quote) || decode_entities(&text).contains(&decode_entities(quote)) {
        return Ok(out);
    }
    if let Some(original) = normalized_span(&text, quote) {
        out["quote"] = json!(original);
        out["evidenceKind"] = json!("normalized_whitespace");
        return Ok(out);
    }
    Err(err("提取引文不在指定原文中，已保留响应等待核对"))
}
