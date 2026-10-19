use crate::{
    contracts::{DesktopError, Result},
    storage::atomic_json,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};
pub const PROVIDERS: [&str; 5] = ["deepseek", "qwenAsr", "qwenTts", "mineru", "evaluation"];
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderConfig {
    pub enabled: bool,
    pub base_url: String,
    pub model: String,
    pub region: String,
    pub voice: String,
    pub language: String,
    pub timeout_seconds: u32,
    pub temperature: f64,
    pub max_tokens: u32,
}
impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: String::new(),
            model: String::new(),
            region: String::new(),
            voice: String::new(),
            language: "en".into(),
            timeout_seconds: 60,
            temperature: 0.7,
            max_tokens: 4096,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioConfig {
    pub input_device_id: String,
    pub output_device_id: String,
    pub master_volume: f64,
    pub tts_volume: f64,
    pub sample_rate: u32,
    pub format: String,
}
impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            input_device_id: String::new(),
            output_device_id: String::new(),
            master_volume: 0.75,
            tts_volume: 0.85,
            sample_rate: 24000,
            format: "wav".into(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NoiseConfig {
    pub profile: String,
    pub volume: f64,
    pub handwriting: bool,
    pub keyboard: bool,
    pub water_pump: bool,
    pub speaking_enabled: bool,
}
impl Default for NoiseConfig {
    fn default() -> Self {
        Self {
            profile: "clean".into(),
            volume: 0.15,
            handwriting: true,
            keyboard: true,
            water_pump: true,
            speaking_enabled: false,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BlenderConfig {
    pub executable_path: String,
    pub source_blend_path: String,
    pub runtime_asset_path: String,
    pub render_scale: f64,
}
impl Default for BlenderConfig {
    fn default() -> Self {
        Self {
            executable_path: "C:/Program Files/Blender Foundation/Blender 5.1/blender.exe".into(),
            source_blend_path: String::new(),
            runtime_asset_path: String::new(),
            render_scale: 1.0,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppConfig {
    pub schema_version: u32,
    pub providers: BTreeMap<String, ProviderConfig>,
    pub audio: AudioConfig,
    pub noise: NoiseConfig,
    pub blender: BlenderConfig,
}
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            schema_version: 1,
            providers: PROVIDERS
                .into_iter()
                .map(|id| (id.to_string(), ProviderConfig::default()))
                .collect(),
            audio: AudioConfig::default(),
            noise: NoiseConfig::default(),
            blender: BlenderConfig::default(),
        }
    }
}
pub fn validate(c: &AppConfig) -> Result<()> {
    let err = |s: &str| DesktopError::new("invalid_config", s);
    if c.schema_version != 1
        || c.providers.len() != 5
        || PROVIDERS.iter().any(|id| !c.providers.contains_key(*id))
    {
        return Err(err("配置版本或服务列表不正确"));
    }
    for (id, p) in &c.providers {
        if p.timeout_seconds < 5
            || p.timeout_seconds > 300
            || !p.temperature.is_finite()
            || !(0.0..=2.0).contains(&p.temperature)
            || p.max_tokens == 0
            || p.max_tokens > 131072
        {
            return Err(err("超时、温度或 Token 上限超出范围"));
        }
        if !p.base_url.is_empty() {
            let url = url::Url::parse(&p.base_url).map_err(|_| err("服务地址格式不正确"))?;
            if !(url.scheme() == "https" || (id == "qwenAsr" && url.scheme() == "wss"))
                || url.host_str().is_none()
                || !url.username().is_empty()
                || url.password().is_some()
                || url.query().is_some()
                || url.fragment().is_some()
            {
                return Err(err(
                    "云服务地址须为 HTTPS（ASR 可用 WSS），不要在 URL 中填写密码、密钥或查询参数",
                ));
            }
        }
        if p.enabled
            && (p.base_url.is_empty()
                || (id != "mineru" && p.model.trim().is_empty()))
        {
            return Err(err("启用服务前请填写地址与模型"));
        }
        for value in [&p.base_url, &p.model, &p.region, &p.voice, &p.language] {
            if value.len() > 2048 || value.chars().any(char::is_control) {
                return Err(err("配置字段过长或包含控制字符"));
            }
        }
    }
    for v in [c.audio.master_volume, c.audio.tts_volume, c.noise.volume] {
        if !v.is_finite() || !(0.0..=1.0).contains(&v) {
            return Err(err("音量必须在 0 到 1 之间"));
        }
    }
    if ![16000, 24000, 48000].contains(&c.audio.sample_rate)
        || !["wav", "mp3", "pcm"].contains(&c.audio.format.as_str())
        || !["clean", "light", "challenge"].contains(&c.noise.profile.as_str())
        || !c.blender.render_scale.is_finite()
        || !(0.5..=2.0).contains(&c.blender.render_scale)
    {
        return Err(err("音频或渲染选项不正确"));
    }
    Ok(())
}
pub fn load(path: &Path) -> Result<AppConfig> {
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let c: AppConfig = serde_json::from_slice(&std::fs::read(path)?)?;
    validate(&c)?;
    Ok(c)
}
pub fn save(path: &Path, c: &AppConfig) -> Result<()> {
    validate(c)?;
    atomic_json(path, c)
}
