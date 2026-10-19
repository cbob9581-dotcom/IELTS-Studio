use super::*;
pub(super) async fn step(
    state: &AppState,
    root: &Path,
    assets: &Path,
    study: &Path,
    j: &mut Value,
) -> Result<()> {
    let dir = jobs::dir(root, j["id"].as_str().unwrap())?;
    let a: Value = serde_json::from_slice(&fs::read(dir.join("attempt.json"))?)?;
    let p = locked(state, |s| {
        papers::load(
            s,
            a["paperId"]
                .as_str()
                .ok_or_else(|| error("作答缺少卷标识"))?,
            a["paperVersion"]
                .as_u64()
                .ok_or_else(|| error("作答缺少卷版本"))?,
        )
    })?;
    let section = a["section"].as_str().unwrap();
    let sec = p["sections"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["section"] == section)
        .unwrap();
    if !j["evaluations"].is_array() {
        j["evaluations"] = json!([]);
    }
    let done = j["evaluations"].as_array().unwrap().len();
    if section == "writing" {
        let qs: Vec<_> = sec["questions"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|q| a["scope"]["part"].as_u64().is_none_or(|n| q["part"] == n))
            .collect();
        if done < qs.len() {
            let q = qs[done];
            let text = a["responses"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["questionId"] == q["id"])
                .and_then(|r| r["value"].as_str())
                .unwrap_or("");
            if text.len() > 60000 {
                return Err(error("写作响应过长"));
            }
            let mut task = q.clone();
            if let Some(image_id) = q["imageAssetId"].as_str() {
                task["visualContext"] = p["assets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|x| x["id"] == image_id)
                    .cloned()
                    .unwrap_or(Value::Null);
            }
            // Generated Task1 table source is retained alongside the paper generation job.
            if q["part"] == 1 {
                if let Some(id) = p["id"].as_str().and_then(|s| s.strip_prefix("ai-")) {
                    if let Ok(job) = jobs::load(assets, id) {
                        if let Ok(plan) = stages(&job["request"]) {
                            if let Some(index) =
                                plan.iter().position(|(s, p)| s == "writing" && *p == 1)
                            {
                                let v: Value = serde_json::from_slice(&fs::read(
                                    jobs::dir(assets, id)?.join(format!("part-{index}.json")),
                                )?)?;
                                task["visualData"] = v["visual"].clone();
                            }
                        }
                    }
                }
            }
            let image_ids=assessment::image_ids(&q);
            let needs_image =
                q["part"] == 1 && task["visualData"].is_null() && !image_ids.is_empty();
            let provider_id = if needs_image {
                "evaluation"
            } else {
                "deepseek"
            };
            let provider = config(state, provider_id)?;
            let key = credentials::read_for_service(provider_id)?;
            let context = serde_json::to_string(
                &json!({"task":task,"candidateResponse":text,"wordCount":text.split_whitespace().count()}),
            )?;
            let content = if needs_image {
                let mut image_parts=vec![];
                for image_id in &image_ids {
                let image = p["assets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|i| i["id"] == image_id.as_str())
                    .ok_or_else(|| error("Task1 图片不存在"))?;
                let bytes = locked(state, |s| {
                    papers::read_asset(
                        s,
                        p["id"].as_str().unwrap(),
                        p["version"].as_u64().unwrap(),
                        image_id,
                    )
                })?;
                if bytes.len() > 7 * 1024 * 1024 {
                    return Err(error("Task1 图片超过视觉模型 Base64 限制"));
                }
                image_parts.push(json!({"type":"image_url","image_url":{"url":format!("data:{};base64,{}",image["mediaType"].as_str().unwrap(),STANDARD.encode(bytes))}}));
                } image_parts.push(json!({"type":"text","text":context})); json!(image_parts)
            } else {
                json!(context)
            };
            let v = cloud::chat(&provider, &key, assessment::WRITING, content, needs_image).await?;
            assessment::validate(&v, "writing", false)?;
            for c in v["criteria"].as_array().unwrap() {
                for e in c["evidence"].as_array().unwrap() {
                    let e = e.as_str().unwrap();
                    if (e == "[no response]" && !text.trim().is_empty())
                        || (e != "[no response]" && !text.contains(e))
                    {
                        return Err(error("写作评测依据不是考生原文引文，未接受该评分"));
                    }
                }
            }
            j["evaluations"].as_array_mut().unwrap().push(json!({"task":q["part"],"questionId":q["id"],"model":provider.model,"wordCount":text.split_whitespace().count(),"result":v}));
            return Ok(());
        }
    } else {
        let turns = a["speaking"]["turns"]
            .as_array()
            .ok_or_else(|| error("没有已保存的口语轮次/录音"))?;
        if turns.is_empty() {
            return Err(error("没有有效口语录音，不能评分"));
        }
        if done < turns.len() {
            let t = &turns[done];
            let take = t["recordingId"]
                .as_str()
                .filter(|s| valid_id(s))
                .ok_or_else(|| error("录音标识无效"))?;
            let bytes = fs::read(
                study
                    .join("recordings")
                    .join(a["id"].as_str().unwrap())
                    .join(format!("{take}.wav")),
            )?;
            speech::validate_wav(&bytes)?;
            let rate = u32::from_le_bytes(bytes[24..28].try_into().unwrap());
            let duration = (bytes.len() - 44) as f64 / (rate.max(1) * 2) as f64;
            let signal = assessment::signal_seconds(&bytes).unwrap_or(0.0);
            if duration < 3.0 || signal < 1.0 {
                j["evaluations"].as_array_mut().unwrap().push(json!({"turnId":t["turnId"],"recordingId":take,"model":"本地证据检查","skipped":true,"signalSeconds":signal,"durationSeconds":duration,"result":{"summary":"此轮语音证据过短，仅保留录音，不单独评分；继续评测其它轮次。","criteria":[],"improvements":[]}}));
                return Ok(());
            }
            let provider = config(state, "evaluation")?;
            let key = credentials::read_for_service("evaluation")?;
            let analysis = assessment::audio_for_model(&bytes)?;
            let content = json!([{"type":"input_audio","input_audio":{"data":format!("data:audio/wav;base64,{}",STANDARD.encode(&analysis)),"format":"wav"}},{"type":"text","text":serde_json::to_string(&json!({"prompt":t["prompt"],"rawAsr":t["text"],"durationSeconds":duration}))?}]);
            let v = cloud::chat(&provider, &key, assessment::SPEAKING, content, true).await?;
            assessment::validate(&v, "speaking", true)?;
            j["evaluations"].as_array_mut().unwrap().push(json!({"turnId":t["turnId"],"recordingId":take,"model":provider.model,"audioSha256":papers::hash(&bytes),"analysisAudioSha256":papers::hash(&analysis),"signalSeconds":signal,"durationSeconds":duration,"result":v}));
            return Ok(());
        }
    }
    let evals = j["evaluations"].as_array().unwrap();
    if evals.is_empty() {
        return Err(error("没有可评测作答"));
    }
    let band = if section == "writing" {
        let t1 = evals.iter().find(|x| x["task"] == 1);
        let t2 = evals.iter().find(|x| x["task"] == 2);
        match (t1, t2) {
            (Some(x), Some(y)) => assessment::writing_band(
                assessment::mean(&x["result"]),
                assessment::mean(&y["result"]),
            ),
            _ => assessment::half(assessment::mean(&evals[0]["result"])),
        }
    } else {
        assessment::speaking_band(evals)?
    };
    let report = json!({"attemptId":a["id"],"section":section,"estimatedBand":band,"label":"未校准 AI 练习参考估分，不是官方 IELTS 成绩","aggregation":if section=="writing"{"四项等权；Task2权重为Task1的两倍，缺一项时仅报告已练习任务"}else{"各轮四项等权，按本地检测的发声时长加权；短轮次不单独评分，不替代整场人工评分"},"evaluations":evals,"createdAt":now_ms()});
    atomic_json(
        &study
            .join("evaluations")
            .join(format!("{}.json", a["id"].as_str().unwrap())),
        &report,
    )?;
    j["report"] = report;
    j["state"] = json!("complete");
    Ok(())
}
