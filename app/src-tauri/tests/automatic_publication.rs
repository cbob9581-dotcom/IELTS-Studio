use ielts_core::cloud::publication;
use serde_json::json;
#[test]
fn a_cached_ai_pass_on_an_undersized_complete_reading_cannot_publish_or_authorize_tts(){
 let mut j=json!({"kind":"generation","state":"review_required","paper":{"sections":[{"section":"reading","paragraphs":[{"part":1,"text":"word ".repeat(600)},{"part":2,"text":"word ".repeat(600)},{"part":3,"text":"word ".repeat(600)}]}]},"validation":{"pass":true,"issues":[]},"audioPending":false});
 j["validationHash"]=json!(ielts_core::papers::hash(&serde_json::to_vec(&j["paper"]).unwrap()));assert!(!publication::ready(&j));assert!(ielts_core::cloud::jobs::authorize_audio(&mut j).is_err());
}
#[test]
fn uploaded_audio_must_decode_before_it_can_make_a_paper_ready(){
 assert!(publication::validate_audio_upload(b"not audio","mp3").is_err());assert!(publication::validate_audio_upload(&[],"wav").is_err());
 let wav=ielts_core::source_library::audio_assets::wav(&vec![0.1;24000],24000);assert!(publication::validate_audio_upload(&wav,"wav").unwrap()>0);
}
#[test]
fn approved_generation_installs_and_persists_without_any_confirmation_field(){
 let tmp=tempfile::tempdir().unwrap();let store=ielts_core::storage::Store::open(ielts_core::contracts::StorageRoots{assets:tmp.path().join("assets").to_string_lossy().into(),study:tmp.path().join("study").to_string_lossy().into()}).unwrap();let root=std::path::Path::new(&store.roots.assets);
 let mut j=ielts_core::cloud::jobs::create(root,"generation",json!({"section":"reading","part":1})).unwrap();let mut paper:serde_json::Value=serde_json::from_str(include_str!("../../test/fixtures/dev-reading.json")).unwrap();paper["status"]=json!("review_required");j["paper"]=paper;j["state"]=json!("review_required");j["validation"]=json!({"pass":true,"issues":[]});j["validationHash"]=json!(ielts_core::papers::hash(&serde_json::to_vec(&j["paper"]).unwrap()));
 assert!(publication::finish(&store,root,&mut j).unwrap());assert_eq!(j["state"],"complete");assert_eq!(ielts_core::cloud::jobs::load(root,j["id"].as_str().unwrap()).unwrap()["state"],"complete");assert!(!publication::finish(&store,root,&mut j).unwrap());
}
#[test]
fn uploaded_listening_audio_is_bound_by_the_system_without_an_answer_edit(){
 let mut j=json!({"kind":"generation","state":"review_required","request":{"section":"listening","part":2},"paper":{"title":"same","assets":[],"sections":[{"section":"listening","audioAssetIds":[]}]},"assets":[{"id":"recording","mediaType":"audio/mpeg"}],"validation":{"pass":true,"issues":[]},"audioPending":true});j["validationHash"]=json!(ielts_core::papers::hash(&serde_json::to_vec(&j["paper"]).unwrap()));
 publication::attach_audio(&mut j,"recording").unwrap();assert_eq!(j["paper"]["sections"][0]["audioAssetIds"],json!(["recording"]));assert_eq!(j["paper"]["sections"][0]["audioParts"]["recording"],2);assert!(publication::ready(&j));
}
#[test]
fn student_does_not_confirm_authored_answers_but_failed_review_and_missing_audio_still_block(){
 let mut j=json!({"kind":"generation","state":"review_required","paper":{"title":"same"},"validation":{"pass":true,"issues":[]},"audioPending":false});
 j["validationHash"]=json!(ielts_core::papers::hash(&serde_json::to_vec(&j["paper"]).unwrap()));assert!(publication::ready(&j));
 j["audioPending"]=json!(true);assert!(!publication::ready(&j));j["audioPending"]=json!(false);j["validation"]["pass"]=json!(false);assert!(!publication::ready(&j));
 j["validation"]["pass"]=json!(true);j["paper"]["title"]=json!("changed");assert!(!publication::ready(&j));
}
