// Local setup helper. Never prints or serializes credentials. The .env is parsed as data, never executed.
use ielts_core::{
    cloud,
    configuration::{self, AppConfig},
    contracts::Result,
    credentials, speech,
};
use serde_json::{json, Value};
use std::{collections::HashMap, fs, path::PathBuf};
fn env_file(path: &str) -> HashMap<String, String> {
    let raw = fs::read_to_string(path).expect("Cannot read user-authorized env file");
    raw.lines()
        .filter_map(|line| {
            let line = line.trim().trim_start_matches('\u{feff}');
            if line.starts_with('#') {
                return None;
            }
            let (k, v) = line
                .strip_prefix("export ")
                .unwrap_or(line)
                .split_once('=')?;
            let v = v.trim();
            let v = if (v.starts_with('"') && v.ends_with('"'))
                || (v.starts_with('\'') && v.ends_with('\''))
            {
                &v[1..v.len() - 1]
            } else {
                v.split(" #").next().unwrap_or(v).trim()
            };
            Some((k.trim().to_string(), v.to_string()))
        })
        .collect()
}
fn install(path: &PathBuf, values: &HashMap<String, String>) -> Result<AppConfig> {
    let mut c = configuration::load(path)?;
    if path.exists() {
        fs::copy(
            path,
            path.with_extension(format!("backup-{}.json", ielts_core::contracts::now_ms())),
        )?;
    }
    for (id, keyname) in [
        ("deepseek", "MODEL_API_KEY"),
        ("qwenAsr", "DASHSCOPE_API_KEY"),
        ("qwenTts", "DASHSCOPE_API_KEY"),
        ("evaluation", "DASHSCOPE_API_KEY"),
        ("mineru", "MINERU_API_KEY"),
    ] {
        if let Some(key) = values.get(keyname).filter(|s| !s.is_empty()) {
            if !credentials::configured(id)? {
                credentials::save(id, key)?;
            }
        }
        let p = c.providers.get_mut(id).unwrap();
        if p.base_url.is_empty() {
            p.base_url = match id {
                "deepseek" => values
                    .get("MODEL_BASE_URL")
                    .cloned()
                    .unwrap_or("https://api.deepseek.com".into()),
                "qwenTts" => values
                    .get("TTS_HTTP_BASE_URL")
                    .cloned()
                    .unwrap_or("https://dashscope.aliyuncs.com/api/v1".into()),
                "qwenAsr" | "evaluation" => {
                    "https://dashscope.aliyuncs.com/compatible-mode/v1".into()
                }
                _ => "https://mineru.net/api/v4".into(),
            };
        }
        if p.model.is_empty() {
            p.model = match id {
                "deepseek" => values
                    .get("MODEL_NAME")
                    .cloned()
                    .unwrap_or("deepseek-flash".into()),
                "qwenTts" => values
                    .get("TTS_STATIC_MODEL")
                    .cloned()
                    .unwrap_or("qwen3-tts-flash".into()),
                "qwenAsr" => "qwen3-asr-flash".into(),
                "evaluation" => "qwen3.8-omni-flash".into(),
                _ => "vlm".into(),
            };
        }
        if id == "qwenTts" && p.voice.is_empty() {
            p.voice = format!(
                "{}|{}|{}",
                values
                    .get("TTS_VOICE_COACH")
                    .map(String::as_str)
                    .unwrap_or("Serena"),
                values
                    .get("TTS_VOICE_ALEX")
                    .map(String::as_str)
                    .unwrap_or("Elias"),
                values.get("TTS_VOICE_SAM").map(String::as_str).unwrap_or("Neil")
            );
        }
        p.enabled = credentials::configured(id)?;
        p.timeout_seconds = if id == "deepseek" || id == "evaluation" {
            300
        } else {
            120
        };
        p.max_tokens = if id == "deepseek" { 16384 } else { 4096 };
    }
    configuration::save(path, &c)?;
    Ok(c)
}
#[tokio::main]
async fn main() {
    let args: Vec<_> = std::env::args().collect();
    let path = PathBuf::from(std::env::var("APPDATA").expect("APPDATA unavailable"))
        .join("local.ielts.studio/app-settings.json");
    let c = if let Some(i) = args.iter().position(|s| s == "--import-env") {
        match install(
            &path,
            &env_file(args.get(i + 1).expect("env path required")),
        ) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Setup failed: {}", e.message);
                std::process::exit(1);
            }
        }
    } else {
        configuration::load(&path).unwrap()
    };
    for (id, p) in &c.providers {
        println!(
            "{}",
            json!({"provider":id,"enabled":p.enabled,"credentialPresent":credentials::configured(id).unwrap_or(false),"model":p.model,"endpoint":p.base_url})
        );
    }
    if !args.iter().any(|s| s == "--probe") {
        return;
    }
    let out = PathBuf::from("../../.tools/cloud-live");
    fs::create_dir_all(&out).unwrap();
    let mut results = Vec::new();
    for id in ["deepseek", "qwenTts", "qwenAsr", "evaluation", "mineru"] {
        let p = c.providers.get(id).unwrap();
        let start = std::time::Instant::now();
        let result:Result<Value>=async {
            let key=credentials::read_for_service(id)?;
            match id {
                "deepseek"=>cloud::chat(p,&key,"Output JSON only: {\"ok\":true}",json!("Connection test. Return JSON."),false).await,
                "qwenTts"=>{let bytes=speech::synthesize(p,&key,"The library opens at nine in the morning.").await?;fs::write(out.join("synthetic-tts.wav"),&bytes)?;Ok(json!({"audioBytes":bytes.len(),"synthetic":true}))},
                "qwenAsr"=>{let bytes=fs::read(out.join("synthetic-tts.wav"))?;let text=speech::recognize(p,&key,&bytes).await?;Ok(json!({"recognized":text,"synthetic":true}))},
                "evaluation"=>{let bytes=fs::read(out.join("synthetic-tts.wav"))?;let content=json!([{"type":"input_audio","input_audio":{"data":format!("data:audio/wav;base64,{}",base64::Engine::encode(&base64::engine::general_purpose::STANDARD,bytes)),"format":"wav"}},{"type":"text","text":"Synthetic connection probe, not a human proficiency test. Output JSON {audioReceived:true,transcript:string}."}]);cloud::chat(p,&key,"Listen to actual audio and return JSON only.",content,true).await},
                _=>{let client=speech::client(p)?;let r=client.get(speech::endpoint(&p.base_url,"extract-results/batch/00000000-0000-0000-0000-000000000000")?).bearer_auth(&key).send().await.map_err(|_|cloud::error("MinerU transport failed"))?;let bytes=speech::bounded(r,1024*1024).await?;let v:Value=serde_json::from_slice(&bytes)?;Ok(json!({"probeCode":v["code"],"scope":"transport only; real upload still required"}))}
            }
        }.await;
        let item = match result {
            Ok(v) => {
                json!({"provider":id,"status":"response_received","result":v,"elapsedMs":start.elapsed().as_millis()})
            }
            Err(e) => {
                json!({"provider":id,"status":"failed","message":e.message,"elapsedMs":start.elapsed().as_millis()})
            }
        };
        println!("{}", item);
        results.push(item);
    }
    ielts_core::storage::atomic_json(&out.join("provider-probes.json"), &results).unwrap();
}
