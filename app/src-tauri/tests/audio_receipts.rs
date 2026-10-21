use ielts_core::{source_library::audio_jobs,cloud,speech::TtsArtifact};
use std::sync::atomic::{AtomicUsize,Ordering};
#[tokio::test]
async fn uncertain_synthesis_is_not_sent_again_and_saved_audio_survives_metadata_loss(){
 let root=tempfile::tempdir().unwrap();let path=root.path().join("segment.audio");let calls=AtomicUsize::new(0);
 let first=audio_jobs::obtain_durable(&path,||Ok(None),|_|Ok(()),||async{calls.fetch_add(1,Ordering::SeqCst);Err(cloud::error("accepted then timeout"))},|_|async{panic!("not a download")}).await;assert!(first.is_err());
 assert!(audio_jobs::obtain_durable(&path,||Ok(None),|_|Ok(()),||async{calls.fetch_add(1,Ordering::SeqCst);Ok(TtsArtifact::Inline(b"duplicate".to_vec()))},|_|async{panic!()}).await.is_err());assert_eq!(calls.load(Ordering::SeqCst),1);
 let cached=root.path().join("saved.audio");let bytes=audio_jobs::obtain_durable(&cached,||Ok(None),|_|Ok(()),||async{Ok(TtsArtifact::Inline(b"recorded audio".to_vec()))},|_|async{panic!()}).await.unwrap();let replay=audio_jobs::obtain_durable(&cached,||panic!("metadata may be lost"),|_|panic!(),||async{panic!("must not synthesize")},|_|async{panic!()}).await.unwrap();assert_eq!(bytes,replay);
}
#[tokio::test]
async fn saved_download_ticket_is_reused_without_resynthesis(){
 let root=tempfile::tempdir().unwrap();let path=root.path().join("remote.audio");let ticket=std::cell::RefCell::new(None::<TtsArtifact>);let calls=AtomicUsize::new(0);
 assert!(audio_jobs::obtain_durable(&path,||Ok(ticket.borrow().clone()),|a|{*ticket.borrow_mut()=Some(a.clone());Ok(())},||async{calls.fetch_add(1,Ordering::SeqCst);Ok(TtsArtifact::Remote{url:"https://example.invalid/audio".into()})},|_|async{Err(cloud::error("download interrupted"))}).await.is_err());
 let audio=audio_jobs::obtain_durable(&path,||Ok(ticket.borrow().clone()),|_|panic!("ticket already saved"),||async{panic!("must not synthesize again")},|url|async move{assert_eq!(url,"https://example.invalid/audio");Ok(b"downloaded".to_vec())}).await.unwrap();
 assert_eq!(audio,b"downloaded");assert_eq!(calls.load(Ordering::SeqCst),1);
}
#[tokio::test]
async fn only_explicit_reset_allows_unknown_synthesis_to_be_sent_again(){
 let root=tempfile::tempdir().unwrap();let path=root.path().join("unknown.audio");
 assert!(audio_jobs::obtain_durable(&path,||Ok(None),|_|Ok(()),||async{Err(cloud::error("unknown response"))},|_|async{panic!()}).await.is_err());
 audio_jobs::reset_response(&path).unwrap();
 assert_eq!(audio_jobs::obtain_durable(&path,||Ok(None),|_|Ok(()),||async{Ok(TtsArtifact::Inline(b"explicit new response".to_vec()))},|_|async{panic!()}).await.unwrap(),b"explicit new response");
}
