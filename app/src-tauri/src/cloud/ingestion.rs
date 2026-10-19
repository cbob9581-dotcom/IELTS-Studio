use super::error;
use crate::{contracts::Result, papers};
use std::{
    fs,
    io::{Cursor, Read},
    path::Path,
};
pub fn namespace_section(v: &mut serde_json::Value, section: &str) -> Result<()> {
    let mut names = std::collections::HashMap::new();
    let sections = v["paper"]["sections"]
        .as_array_mut()
        .ok_or_else(|| error("提取结果缺少科目列表"))?;
    for s in sections {
        if s["section"] != section {
            return Err(error("提取结果混入其他科目"));
        }
        if let Some(paragraphs) = s["paragraphs"].as_array_mut() {
            for (i, p) in paragraphs.iter_mut().enumerate() {
                p["id"] = serde_json::json!(format!("{section}-p{}", i + 1));
            }
        }
        let questions = s["questions"]
            .as_array_mut()
            .ok_or_else(|| error("提取结果缺少题目列表"))?;
        for (i, q) in questions.iter_mut().enumerate() {
            let old = q["id"].as_str().unwrap_or("").to_string();
            let id = format!("{section}-q{}", i + 1);
            names.insert(old, id.clone());
            q["id"] = serde_json::json!(id);
        }
    }
    if let Some(mapping) = v["mapping"].as_array_mut() {
        for m in mapping {
            if let Some(id) = m["questionId"].as_str().and_then(|id| names.get(id)) {
                m["questionId"] = serde_json::json!(id);
            }
        }
    }
    Ok(())
}
pub fn safe_entry(p: &str) -> bool {
    papers::safe_path(p)
        && p.split('/').all(|s| {
            if s.ends_with(['.', ' ']) {
                return false;
            }
            let stem = s.split('.').next().unwrap_or("").to_ascii_uppercase();
            ![
                "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
                "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8",
                "LPT9",
            ]
            .contains(&stem.as_str())
        })
}
pub fn extract(bytes: &[u8], dest: &Path) -> Result<()> {
    let mut z =
        zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_| error("MinerU 返回的 ZIP 无效"))?;
    if z.len() > 4000 {
        return Err(error("解析包文件过多"));
    }
    let mut total = 0u64;
    for i in 0..z.len() {
        let mut f = z.by_index(i).map_err(|_| error("读取解析包失败"))?;
        let entry_name = f
            .name()
            .map_err(|_| error("压缩包文件名编码错误"))?
            .into_owned();
        let name = entry_name.trim_end_matches('/');
        if !safe_entry(name) || f.unix_mode().is_some_and(|m| m & 0o170000 == 0o120000) {
            return Err(error("解析包包含危险路径或符号链接"));
        }
        total += f.size();
        if total > 300 * 1024 * 1024 || f.size() > 64 * 1024 * 1024 {
            return Err(error("解压后资源过大"));
        }
        let path = dest.join(name);
        if f.is_dir() {
            fs::create_dir_all(path)?;
            continue;
        }
        fs::create_dir_all(path.parent().unwrap())?;
        let mut data = Vec::new();
        f.by_ref()
            .take(64 * 1024 * 1024 + 1)
            .read_to_end(&mut data)?;
        if data.len() > 64 * 1024 * 1024 {
            return Err(error("解压资源超限"));
        }
        fs::write(path, data)?;
    }
    Ok(())
}
pub fn markdown(root: &Path) -> Result<String> {
    fn collect(p: &Path, out: &mut Vec<std::path::PathBuf>) -> Result<()> {
        for e in fs::read_dir(p)? {
            let p = e?.path();
            if p.is_dir() {
                collect(&p, out)?;
            } else if p.extension().is_some_and(|x| x == "md") {
                out.push(p);
            }
        }
        Ok(())
    }
    let mut paths = Vec::new();
    collect(root, &mut paths)?;
    paths.sort();
    let preferred = paths
        .iter()
        .find(|p| p.file_name().is_some_and(|x| x == "full.md"))
        .or(paths.first())
        .ok_or_else(|| error("解析包中没有 Markdown"))?;
    if fs::metadata(preferred)?.len() > 2 * 1024 * 1024 {
        return Err(error("解析文本过大，请分卷导入"));
    }
    Ok(fs::read_to_string(preferred)?)
}
pub fn verify_answer_sources(p: &serde_json::Value, source: &str) -> Result<()> {
    for section in p["sections"].as_array().ok_or_else(|| error("缺少科目"))? {
        for q in section["questions"]
            .as_array()
            .ok_or_else(|| error("缺少题目"))?
        {
            if ["writing_task", "speaking_prompt"].contains(&q["kind"].as_str().unwrap_or("")) {
                continue;
            }
            let quotes: Vec<_> = q["evidence"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|e| e.as_str()?.strip_prefix("ANSWER_SOURCE:"))
                .map(str::trim)
                .filter(|s| {
                    if s.len() < 3 {
                        return false;
                    }
                    if source.contains(*s) {
                        return true;
                    }
                    // Models sometimes quote an otherwise exact excerpt for presentation.
                    // Only remove one matched outer pair; never normalize or invent its contents.
                    [('"', '"'), ('“', '”'), ('‘', '’'), ('`', '`')]
                        .iter()
                        .any(|(open, close)| {
                            s.strip_prefix(*open)
                                .and_then(|x| x.strip_suffix(*close))
                                .map(str::trim)
                                .is_some_and(|inner| inner.len() >= 3 && source.contains(inner))
                        })
                })
                .collect();
            if quotes.is_empty() {
                return Err(error(format!("题目 {} 缺少原资料中的答案键引文。请在 evidence 添加 ANSWER_SOURCE: 后跟原文精确片段；资料没有答案时不能发布客观题。",q["id"])));
            }
        }
    }
    Ok(())
}
