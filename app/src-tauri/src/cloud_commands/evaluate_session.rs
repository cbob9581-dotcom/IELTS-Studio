use super::*;
pub(super) async fn step(
    state: &AppState,
    root: &Path,
    study: &Path,
    j: &mut Value,
    a: &Value,
    paper: &Value,
) -> Result<()> {
    let dir = jobs::dir(root, j["id"].as_str().unwrap())?;
    let turns = a["speaking"]["turns"]
        .as_array()
        .filter(|t| !t.is_empty())
        .ok_or_else(|| error("没有已保存的口语回答，无法进行整场评测"))?;
    let provider = config(state, "evaluation")?;
    let key = credentials::read_for_service("evaluation")?;
    if !assessment::session_audio_supported(&provider.model, turns.len()) {
        return Err(error("当前评测模型不支持多段录音整场理解。请在设置选用支持多音频的评测模型（如 qwen3.8-omni-flash），原始录音和已有反馈保留；尚未发送评测请求"));
    }
    let attempt = a["id"]
        .as_str()
        .filter(|s| valid_id(s))
        .ok_or_else(|| error("作答标识无效"))?;
    let mut content = vec![];
    let mut recordings = vec![];
    let mut context = vec![];
    let mut bytes_total = 0usize;
    for turn in turns {
        let take = turn["recordingId"]
            .as_str()
            .filter(|s| valid_id(s))
            .ok_or_else(|| error("录音标识无效"))?;
        let bytes = fs::read(
            study
                .join("recordings")
                .join(attempt)
                .join(format!("{take}.wav")),
        )?;
        speech::validate_wav(&bytes)?;
        let rate = u32::from_le_bytes(bytes[24..28].try_into().unwrap());
        let duration = (bytes.len() - 44) as f64 / (rate as f64 * 2.);
        let signal = assessment::signal_seconds(&bytes).unwrap_or(0.);
        let question = paper["sections"]
            .as_array()
            .into_iter()
            .flatten()
            .flat_map(|s| s["questions"].as_array().into_iter().flatten())
            .find(|q| q["id"] == turn["questionId"]);
        let meta = json!({"turnId":turn["turnId"],"recordingId":take,"part":question.map(|q|q["part"].clone()),"prompt":turn["prompt"],"rawAsr":turn["text"],"durationSeconds":duration,"signalSeconds":signal});
        context.push(meta.clone());
        // Keep short/silent turns in the context; they cannot block other useful speech.
        if duration < 0.5 || signal < 0.3 {
            continue;
        }
        let analysis = assessment::audio_for_model(&bytes)?;
        bytes_total += analysis.len();
        if bytes_total > 48 * 1024 * 1024 {
            return Err(error("整场分析音频超过本次上传预算，请按 Part 评测或使用24kHz录音；原录音保留，未发送云请求"));
        }
        content.push(json!({"type":"text","text":serde_json::to_string(&meta)?}));
        content.push(json!({"type":"input_audio","input_audio":{"data":format!("data:audio/wav;base64,{}",STANDARD.encode(&analysis)),"format":"wav"}}));
        let mut saved = meta;
        saved["audioSha256"] = json!(papers::hash(&bytes));
        recordings.push(saved);
    }
    let signal: f64 = recordings
        .iter()
        .map(|r| r["signalSeconds"].as_f64().unwrap_or(0.))
        .sum();
    if signal < 30. {
        return Err(error(
            "检测到的发声总时长不足30秒；已保留全部录音，不发送整场评分请求",
        ));
    }
    content.push(json!({"type":"text","text":serde_json::to_string(&json!({"scope":a["scope"],"allTurns":context,"instruction":"Assess the actual available scope. A selected training Part is valid and must not be treated as a complete official exam. All timestamps are local to the identified recording."}))?}));
    jobs::operation(
        root,
        j,
        "assessment_session",
        "根据整场实际录音评估四项表现",
        0,
        1,
        "场",
    )?;
    let result = cloud::chat_recorded(
        &provider,
        &key,
        assessment::SPEAKING_SESSION,
        json!(content),
        true,
        Some(&dir.join("speaking-session.response.txt")),
    )
    .await?;
    let band = assessment::validate_session(&result, &json!(recordings))?;
    if !j["evaluations"].as_array().is_none_or(|e| e.is_empty()) {
        j["previousRoundEvaluations"] = j["evaluations"].clone();
    }
    j["evaluations"] = json!([{"turnId":"session","protocol":"whole-session-v1","model":provider.model,"recordings":recordings,"durationSeconds":context.iter().map(|r|r["durationSeconds"].as_f64().unwrap_or(0.)).sum::<f64>(),"signalSeconds":signal,"result":result}]);
    let report = json!({"attemptId":a["id"],"protocol":"whole-session-v1","section":"speaking","estimatedBand":band,"label":"未校准 AI 练习参考估分，不是官方 IELTS 成绩","aggregation":"模型依据整场可用录音及上下文评估四项表现，四项等权；未按各轮时长平均。可点击录音核对证据。","evaluations":j["evaluations"],"createdAt":now_ms()});
    atomic_json(
        &study.join("evaluations").join(format!("{attempt}.json")),
        &report,
    )?;
    j["report"] = report;
    j["state"] = json!("complete");
    Ok(())
}
