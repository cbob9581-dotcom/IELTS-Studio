use ielts_core::source_library::audio_jobs;
use serde_json::json;
#[test]
fn cache_key_changes_only_for_spoken_parameters(){
 let mut p=ielts_core::configuration::ProviderConfig::default();p.model="qwen3-tts-flash".into();p.voice="Aiden".into();
 let a=audio_jobs::segment_key("Hello",&p);assert_eq!(a,audio_jobs::segment_key("Hello",&p));assert_ne!(a,audio_jobs::segment_key("Hello!",&p));p.voice="Cherry".into();assert_ne!(a,audio_jobs::segment_key("Hello",&p));
}
#[test]
fn missing_only_skips_validated_cached_segments(){
 let draft=ielts_core::source_library::transcripts::parse("JAN: Hello. PETER: Hi.","user","Test 1",1).unwrap();
 let done=json!({"t1":{"key":"old","ready":true}});
 assert_eq!(audio_jobs::pending(&draft,&done).unwrap().len(),1);
}
#[test]
fn corrupt_cache_is_not_reused(){
 let dir=tempfile::tempdir().unwrap();let path=dir.path().join("key.wav");std::fs::write(&path,b"broken").unwrap();assert!(!audio_jobs::cached(&path));
}
#[tokio::test]
async fn authentication_error_is_not_retried(){
 let count=std::sync::atomic::AtomicUsize::new(0);
 let result=audio_jobs::with_retry(||{count.fetch_add(1,std::sync::atomic::Ordering::Relaxed);async {Err(ielts_core::contracts::DesktopError::new("speech","云服务 HTTP 401：密钥无效"))}}).await;
 assert!(result.is_err());assert_eq!(count.load(std::sync::atomic::Ordering::Relaxed),1);
}
#[test]
fn repeated_text_keys_create_only_one_request(){
 assert_eq!(audio_jobs::unique_requests(&["a".into(),"a".into(),"b".into(),"c".into()],3),vec![0,2,3]);
}
#[tokio::test]
async fn failed_download_resumes_existing_artifact_without_second_post(){
 use ielts_core::speech::TtsArtifact;
 use std::sync::{Arc,Mutex,atomic::{AtomicUsize,Ordering}};
 let saved=Arc::new(Mutex::new(None::<TtsArtifact>));let posts=AtomicUsize::new(0);let downloads=AtomicUsize::new(0);
 for _ in 0..2 {let result=audio_jobs::obtain(||Ok(saved.lock().unwrap().clone()),|a|{*saved.lock().unwrap()=Some(a.clone());Ok(())},||async{posts.fetch_add(1,Ordering::Relaxed);Ok(TtsArtifact::Remote{url:"https://example.com/signed".into()})},|_|async{if downloads.fetch_add(1,Ordering::Relaxed)==0{Err(ielts_core::contracts::DesktopError::new("speech","下载中断"))}else{Ok(vec![1,2,3])}}).await;if downloads.load(Ordering::Relaxed)==1{assert!(result.is_err());}else{assert_eq!(result.unwrap(),vec![1,2,3]);}}
 assert_eq!(posts.load(Ordering::Relaxed),1);assert_eq!(downloads.load(Ordering::Relaxed),2);
}
