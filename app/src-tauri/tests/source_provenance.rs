use ielts_core::source_library::{provenance,matching};
use serde_json::json;
#[test]
fn line_positions_in_legacy_blockids_are_resolved_only_when_the_actual_lines_exist(){
 let blocks=json!([{"id":"pdf-page-120","text":"Answer Key\n1 population\n2 suburbs"}]);let e=provenance::resolve(&json!({"blockIds":["pdf-page-120-l2"],"quote":"model retyped text"}),&blocks).unwrap();assert_eq!(e["blockIds"],json!(["pdf-page-120"]));assert_eq!(e["quote"],"1 population");assert_eq!(e["sourceVerified"],true);assert!(provenance::resolve(&json!({"blockIds":["pdf-page-120-l99"],"quote":"1 population"}),&blocks).is_err());
}
#[test]
fn source_line_ids_bind_actual_text_and_never_trust_a_model_retyped_quote(){
 let blocks=json!([{"id":"b1","page":3,"text":"1. river\n2. BLACK"}]);
 let shown=provenance::for_model(&blocks).unwrap();assert_eq!(shown[0]["sourceLines"][1]["id"],"b1-l2");
 let entry=provenance::resolve(&json!({"sourceLineIds":["b1-l2"],"quote":"made up"}),&blocks).unwrap();
 assert_eq!(entry["quote"],"2. BLACK");assert_eq!(entry["blockIds"],json!(["b1"]));matching::verify_entry(&entry,&blocks).unwrap();
 assert!(provenance::resolve(&json!({"sourceLineIds":["b2-l1"]}),&blocks).is_err());
}
#[test]
fn legacy_quotes_may_vary_in_whitespace_but_not_in_words_or_answers(){
 let blocks=json!([{"id":"b1","text":"1.\n river   bank\n2. BLACK"}]);
 let entry=provenance::resolve(&json!({"blockIds":["b1"],"quote":"1. river bank"}),&blocks).unwrap();assert!(entry["quote"].as_str().unwrap().contains("\n"));
 assert!(provenance::resolve(&json!({"blockIds":["b1"],"quote":"1. river banks"}),&blocks).is_err());
}
#[test]
fn image_transcription_keeps_registered_image_provenance_and_requires_confirmation(){
 let blocks=json!([{"id":"b1","page":9,"text":"","imageAssetId":"answer-page-9"}]);
 let e=provenance::resolve(&json!({"sourceImageId":"answer-page-9","transcription":"1. BLACK","numbers":[1]}),&blocks).unwrap();
 assert_eq!(e["evidenceKind"],"visual_transcription");assert_eq!(e["sourceVerified"],false);assert_eq!(e["quote"],"1. BLACK");
 assert!(provenance::resolve(&json!({"sourceImageId":"other-book","transcription":"1. BLACK"}),&blocks).is_err());
 let mut p=json!({"units":[{"test":"Test 1","section":"listening","role":"questions","result":{"section":{"questions":[{"id":"q1","number":1,"part":1}]}}}],"entries":[{"id":"a","kind":"answers","test":"Test 1","section":"listening","numbers":[1],"answer":{"accepted":["BLACK"]},"evidenceKind":"visual_transcription","sourceVerified":false}]});
 matching::reconcile(&mut p).unwrap();assert!(p["units"][0]["result"]["section"]["questions"][0].get("answer").is_none());assert_eq!(p["entries"][0]["state"],"source_review");
}
