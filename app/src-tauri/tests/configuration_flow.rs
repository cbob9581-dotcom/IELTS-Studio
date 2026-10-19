use ielts_core::configuration::{load, save, validate, AppConfig};
#[test]
fn settings_roundtrip_and_invalid_settings_preserve_previous() {
    let t = tempfile::tempdir().unwrap();
    let path = t.path().join("配置 settings.json");
    let mut cfg = AppConfig::default();
    cfg.providers.get_mut("deepseek").unwrap().model = "test-model".into();
    save(&path, &cfg).unwrap();
    assert_eq!(
        load(&path).unwrap().providers["deepseek"].model,
        "test-model"
    );
    cfg.providers.get_mut("deepseek").unwrap().base_url = "http://unsafe.example".into();
    assert!(save(&path, &cfg).is_err());
    assert_eq!(load(&path).unwrap().providers["deepseek"].base_url, "");
    let value = serde_json::to_string(&cfg).unwrap();
    assert!(!value.contains("apiKey"));
}
#[test]
fn rejects_secret_in_url_and_invalid_volumes() {
    let mut cfg = AppConfig::default();
    cfg.providers.get_mut("qwenTts").unwrap().base_url =
        "https://user:password@example.com/v1".into();
    assert!(validate(&cfg).is_err());
    cfg.providers.get_mut("qwenTts").unwrap().base_url = "".into();
    cfg.audio.master_volume = 2.0;
    assert!(validate(&cfg).is_err());
}
