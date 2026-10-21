use ielts_core::speech_cache;
#[tokio::test]
async fn fixed_examiner_audio_is_reused_and_uncertain_tts_requires_an_explicit_retry(){
 let root=tempfile::tempdir().unwrap();let key="a".repeat(64);let original=b"RIFF0000WAVEaudio".to_vec();
 let first=speech_cache::obtain(root.path(),&key,false,||async{Ok(original.clone())}).await.unwrap();
 let second=speech_cache::obtain(root.path(),&key,false,||async{panic!("cached examiner text must not consume TTS")}).await.unwrap();assert_eq!(first,second);
 let unknown="b".repeat(64);assert!(speech_cache::obtain(root.path(),&unknown,false,||async{Err(ielts_core::cloud::error("unknown TTS result"))}).await.is_err());
 assert!(speech_cache::obtain(root.path(),&unknown,false,||async{panic!("uncertain TTS must not retry by itself")}).await.is_err());
 assert!(speech_cache::obtain(root.path(),&unknown,true,||async{Ok(original.clone())}).await.is_ok());
}
