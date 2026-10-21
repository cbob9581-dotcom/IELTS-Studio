use ielts_core::{configuration::ProviderConfig, speech};
#[test]
fn deepseek_fast_requests_do_not_spend_json_or_question_budget_on_hidden_thinking() {
    let mut p = ProviderConfig::default();
    p.base_url = "https://api.deepseek.com".into();
    p.model = "deepseek-flash".into();
    let mut body = serde_json::json!({"max_tokens":512});
    speech::fast_mode(&p, &mut body);
    assert_eq!(body["thinking"]["type"], "disabled");
    p.base_url = "https://dashscope.aliyuncs.com/compatible-mode/v1".into();
    let mut body = serde_json::json!({});
    speech::fast_mode(&p, &mut body);
    assert!(body.get("thinking").is_none());
}
#[test]
fn qwen_payloads_keep_raw_audio_and_configured_voice() {
    let mut p = ProviderConfig::default();
    p.model = "qwen3-tts-flash".into();
    p.voice = "Cherry".into();
    let t = speech::tts_payload(&p, "Please begin.").unwrap();
    assert_eq!(t["input"]["voice"], "Cherry");
    assert_eq!(t["input"]["language_type"], "English");
    let a = speech::asr_payload(&p, "UklGRg==");
    assert_eq!(a["asr_options"]["enable_itn"], false);
    assert_eq!(
        a["messages"][0]["content"][0]["input_audio"]["data"],
        "data:audio/wav;base64,UklGRg=="
    );
    assert!(speech::tts_payload(&p, &"a".repeat(601)).is_err());
}
#[test]
fn qwen_omni_fast_analysis_and_shared_speech_cache_use_the_actual_provider_parameters(){
 let mut p=ProviderConfig::default();p.base_url="https://dashscope.aliyuncs.com/compatible-mode/v1".into();p.model="qwen3.8-omni-flash".into();
 let mut body=serde_json::json!({});speech::fast_mode(&p,&mut body);assert_eq!(body["reasoning_effort"],"none");
 let first=speech::speech_cache_key(&p,"Hello");p.voice="New voice".into();assert_ne!(first,speech::speech_cache_key(&p,"Hello"));
}
#[test]
fn endpoint_normalization_never_drops_workspace_path_or_accepts_ws() {
    assert_eq!(
        speech::endpoint("https://api.deepseek.com/v1", "chat/completions").unwrap(),
        "https://api.deepseek.com/v1/chat/completions"
    );
    assert_eq!(
        speech::endpoint(
            "https://api.deepseek.com/v1/chat/completions",
            "chat/completions"
        )
        .unwrap(),
        "https://api.deepseek.com/v1/chat/completions"
    );
    assert!(speech::endpoint(
        "wss://dashscope.aliyuncs.com/api-ws/v1/realtime",
        "chat/completions"
    )
    .is_err());
    assert!(speech::endpoint("https://u:password@example.com", "chat/completions").is_err());
}

#[test]
fn recovering_segments_keeps_pcm_order_and_rejects_mixed_rates() {
    fn wav(rate: u32, sample: i16) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend(b"RIFF");
        b.extend(38u32.to_le_bytes());
        b.extend(b"WAVEfmt ");
        b.extend(16u32.to_le_bytes());
        b.extend(1u16.to_le_bytes());
        b.extend(1u16.to_le_bytes());
        b.extend(rate.to_le_bytes());
        b.extend((rate * 2).to_le_bytes());
        b.extend(2u16.to_le_bytes());
        b.extend(16u16.to_le_bytes());
        b.extend(b"data");
        b.extend(2u32.to_le_bytes());
        b.extend(sample.to_le_bytes());
        b
    }
    let a = wav(24000, 7);
    let b = wav(24000, 9);
    let merged = speech::join_waves(&[a.clone(), b]).unwrap();
    speech::validate_wav(&merged).unwrap();
    assert_eq!(&merged[44..], &[7, 0, 9, 0]);
    assert!(speech::join_waves(&[a, wav(48000, 9)]).is_err());
}
