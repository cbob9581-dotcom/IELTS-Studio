pub mod assessment;
pub mod generation;
pub mod generation_workflow;
pub mod ingestion;
pub mod jobs;
pub mod repair;
pub mod request;
pub mod visuals;
pub mod review;
pub mod publication;
use crate::{
    configuration::ProviderConfig,
    contracts::{DesktopError, Result},
    speech,
};
use serde_json::{json, Value};
pub fn error(s: impl ToString) -> DesktopError {
    DesktopError::new("cloud", s)
}
pub fn parse_sse(raw: &str) -> Result<Value> {
    let mut text = String::new();
    let mut finished = false;
    for line in raw.lines() {
        let Some(data) = line.strip_prefix("data:") else {
            continue;
        };
        let data = data.trim();
        if data == "[DONE]" {
            continue;
        }
        let v: Value = serde_json::from_str(data).map_err(|_| error("流式响应 JSON 损坏"))?;
        if v.get("error").is_some() {
            return Err(error("云模型流式请求失败，请检查模型权限"));
        }
        if let Some(s) = v["choices"][0]["delta"]["content"].as_str() {
            text.push_str(s);
        }
        if v["choices"][0]["finish_reason"] == "stop" {
            finished = true;
        }
    }
    if !finished {
        return Err(error(
            "模型输出不完整或达到 Token 上限；保留已有任务，请调整配置后重试该步骤",
        ));
    }
    parse_json(&text)
}
pub fn parse_json(raw: &str) -> Result<Value> {
    let raw = raw.trim();
    let raw = raw
        .strip_prefix("```json")
        .or_else(|| raw.strip_prefix("```"))
        .and_then(|s| s.trim_end().strip_suffix("```"))
        .unwrap_or(raw)
        .trim();
    let v: Value = serde_json::from_str(raw).map_err(|_| {
        error("模型输出不是完整 JSON；未发布内容，请检查当前步骤的任务记录后决定是否重试")
    })?;
    if !v.is_object() {
        return Err(error("模型需要返回 JSON 对象"));
    }
    Ok(v)
}
pub async fn chat(
    p: &ProviderConfig,
    key: &str,
    system: &str,
    content: Value,
    stream: bool,
) -> Result<Value> {
    chat_recorded(p, key, system, content, stream, None).await
}
pub async fn chat_recorded(
    p: &ProviderConfig,
    key: &str,
    system: &str,
    content: Value,
    stream: bool,
    record: Option<&std::path::Path>,
) -> Result<Value> {
    if p.model.trim().is_empty() {
        return Err(error("请配置模型 ID"));
    }
    let mut body = json!({"model":p.model,"stream":stream,"temperature":p.temperature,"max_tokens":p.max_tokens,
        "messages":[{"role":"system","content":system},{"role":"user","content":content}]});
    if stream {
        body["modalities"] = json!(["text"]);
    } else {
        body["response_format"] = json!({"type":"json_object"});
    }
    speech::fast_mode(p, &mut body);
    let c = speech::client(p)?;
    let send=||async {let r = c
        .post(speech::endpoint(&p.base_url, "chat/completions")?)
        .bearer_auth(key)
        .json(&body)
        .send()
        .await
        .map_err(|_| error("模型请求失败或超时；请检查任务后再决定是否重试，以免重复计费"))?;
    speech::bounded(r, 8 * 1024 * 1024).await};
    let bytes=if let Some(path)=record{request::recorded(path,&json!({"endpoint":p.base_url,"request":body}),send).await?}else{send().await?};
    if stream {
        return parse_sse(std::str::from_utf8(&bytes).map_err(|_| error("模型响应编码错误"))?);
    }
    let v: Value = serde_json::from_slice(&bytes).map_err(|_| error("云服务响应格式错误"))?;
    if v["choices"][0]["finish_reason"] == "length"
        && v["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .is_empty()
    {
        return Err(error("模型把输出额度用在思考过程，未返回试题。请使用新版的 DeepSeek 快速请求模式，再继续本任务；已有部分仍保留。"));
    }
    if v["choices"][0]["finish_reason"] != "stop" {
        return Err(error(
            "输出截断或未正常结束，请提高 Token 上限后重试当前步骤",
        ));
    }
    parse_json(
        v["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| error("模型没有返回文本 JSON"))?,
    )
}
pub fn signed_url(raw: &str) -> Result<url::Url> {
    let u = url::Url::parse(raw).map_err(|_| error("下载地址无效"))?;
    if u.scheme() != "https"
        || !u.username().is_empty()
        || u.password().is_some()
        || u.host_str().is_none()
    {
        return Err(error("仅允许 HTTPS 资源下载"));
    }
    if let Ok(ip) = u.host_str().unwrap().parse::<std::net::IpAddr>() {
        if ip.is_loopback() || ip.is_unspecified() {
            return Err(error("资源地址不能是本机"));
        }
    }
    Ok(u)
}
