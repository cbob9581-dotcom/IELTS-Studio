use super::err;
use crate::contracts::Result;
use serde_json::{json, Value};
fn text(v: &Value) -> String {
    if let Some(s) = v.as_str() {
        return s.into();
    }
    if let Some(a) = v.as_array() {
        return a.iter().map(text).collect::<Vec<_>>().join("\n");
    }
    if v.is_object() {
        for k in ["text", "content", "table_body"] {
            if let Some(x) = v.get(k) {
                return text(x);
            }
        }
    }
    String::new()
}
pub fn normalize(v: &Value) -> Result<Vec<Value>> {
    let mut raw = vec![];
    if let Some(pages) = v["pages"].as_array() {
        for page in pages {
            for b in page["blocks"]
                .as_array()
                .ok_or_else(|| err("版面 pages 缺少 blocks"))?
            {
                raw.push((page["page_idx"].as_u64().unwrap_or(0), b.clone(), false));
            }
        }
    } else if let Some(items) = v.as_array() {
        for b in items {
            if let Some(page) = b.as_array() {
                for item in page {
                    raw.push((item["page_idx"].as_u64().unwrap_or(0), item.clone(), true));
                }
            } else {
                raw.push((b["page_idx"].as_u64().unwrap_or(0), b.clone(), true));
            }
        }
    } else {
        return Err(err(
            "未识别此 MinerU 版面格式，请保留原件并使用原文预览确认",
        ));
    }
    let mut out = vec![];
    for (page, b, legacy) in raw {
        let mut t = text(&b);
        for k in ["image_caption", "table_caption", "table_footnote"] {
            if let Some(v) = b.get(k) {
                t.push_str(&format!("\n{}", text(v)));
            }
        }
        let image = b["img_path"]
            .as_str()
            .or(b["image_path"].as_str())
            .unwrap_or("");
        let bbox = if legacy {
            b["bbox"]
                .as_array()
                .filter(|a| a.len() == 4)
                .map(|a| {
                    json!(a
                        .iter()
                        .map(|n| n.as_f64().unwrap_or(0.) / 1000.)
                        .collect::<Vec<_>>())
                })
                .unwrap_or(Value::Null)
        } else {
            Value::Null
        };
        let mut normalized=json!({"id":format!("b{}",out.len()+1),"page":page+1,"type":b["type"].as_str().unwrap_or("text"),"text":t,"imagePath":image,"bbox":bbox,"rawBbox":b["bbox"],"coordinateSystem":if legacy{"normalized1000"}else{"unknown"}});
        normalized["auxiliaryImage"]=json!(is_auxiliary(&normalized));out.push(normalized);
    }
    if out.is_empty() {
        return Err(err("版面中没有可提取的内容"));
    }
    Ok(out)
}
pub fn is_auxiliary(b:&Value)->bool{
 // Size/position cannot distinguish a QR code from a real small legend.
 b["auxiliaryImage"]==true&&b["auxiliaryConfirmed"]==true
}
pub fn requires_visual(b:&Value)->bool{
 b["imageAssetId"].is_string()&&!is_auxiliary(b)&&!(b["type"]=="table"&&b["text"].as_str().unwrap_or("").contains("<table"))
}
pub fn plain(s: &str) -> Vec<Value> {
    s.split("\n\n").filter(|s|!s.trim().is_empty()).enumerate().map(|(n,s)|json!({"id":format!("b{}",n+1),"page":null,"type":"text","text":s,"imagePath":"","bbox":null})).collect()
}
/// Non-test front matter remains a source range instead of failing the whole book.
pub fn scope(span:&Value)->Option<Value>{
 let test=span["test"].as_str()?.trim();if test.is_empty(){return None}
 let test=if test.to_ascii_lowercase().starts_with("test") {let number=test[4..].trim().parse::<u32>().ok()?;format!("Test {number}")}else if let Ok(number)=test.parse::<u32>(){format!("Test {number}")}else{test.to_string()};
 let section=span["section"].as_str()?.to_ascii_lowercase();
 if !["listening","reading","writing","speaking"].contains(&section.as_str()){return None}
 let role=span["role"].as_str()?;if !["questions","answers","explanations","essays","transcript"].contains(&role){return None}
 let part=span["part"].as_u64().or_else(||if role!="questions"{Some(1)}else{None})?;
 let max=match section.as_str(){"writing"=>2,"listening"=>4,_=>3};if part==0||part>max{return None}
 let mut out=span.clone();out["test"]=json!(test);out["section"]=json!(section);out["part"]=json!(part);Some(out)
}
