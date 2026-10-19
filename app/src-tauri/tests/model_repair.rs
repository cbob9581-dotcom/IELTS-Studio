use ielts_core::cloud::repair;
use serde_json::json;
#[test]
fn feedback_preserves_original_request_rejected_output_and_exact_failure() {
 let v=repair::feedback(&json!({"scope":"reading Test 2","source":"1. solar"}),&json!({"entries":[{"text":"solar"}]}),"entries[0].answer.accepted 缺失");
 assert_eq!(v["originalRequest"]["scope"],"reading Test 2");
 assert_eq!(v["rejectedOutput"]["entries"][0]["text"],"solar");
 assert_eq!(v["validationErrors"][0],"entries[0].answer.accepted 缺失");
 assert!(v["instruction"].as_str().unwrap().contains("Do not invent"));
}

#[tokio::test]
async fn rejected_correction_is_reused_and_never_triggers_an_unbounded_retry() {
 let tmp=tempfile::tempdir().unwrap();let path=tmp.path().join("part.json");
 std::fs::write(path.with_extension("repair.json"),r#"{"answer":null}"#).unwrap();
 let provider=ielts_core::configuration::ProviderConfig::default();
 let original=json!({"answer":null});
 std::fs::write(path.with_extension("repair-attempt.json"),serde_json::to_vec(&json!({"originalHash":ielts_core::papers::hash(&serde_json::to_vec(&original).unwrap()),"state":"received"})).unwrap()).unwrap();
 for _ in 0..2 {
  let error=repair::checked(&provider,"unused","system",json!("original scope"),false,&path,json!({"answer":null}),|v|{
   if v["answer"].is_string(){Ok(())}else{Err(ielts_core::cloud::error("answer 字段缺失"))}
  }).await.unwrap_err();
  assert!(error.message.contains("1 次"));
 }
 let report:serde_json::Value=serde_json::from_slice(&std::fs::read(path.with_extension("feedback.json")).unwrap()).unwrap();
 assert_eq!(report["validationErrors"][0],"answer 字段缺失");
 assert!(!path.with_extension("repair-response.txt").exists());
}

#[test]
fn malformed_model_json_can_be_returned_for_repair_but_network_errors_cannot() {
 let tmp=tempfile::tempdir().unwrap();let f=tmp.path().join("response.txt");
 std::fs::write(&f,serde_json::to_vec(&json!({"choices":[{"message":{"content":"{\"questions\":"}}]})).unwrap()).unwrap();
 assert!(repair::recoverable_output(&ielts_core::cloud::error("模型输出不是完整 JSON"),&f).unwrap().is_some());
 assert!(repair::recoverable_output(&ielts_core::cloud::error("模型请求失败或超时"),&f).unwrap().is_none());
}

#[test]
fn malformed_streaming_vision_output_is_recoverable() {
 let tmp=tempfile::tempdir().unwrap();let f=tmp.path().join("stream.txt");
 std::fs::write(&f,"data: {\"choices\":[{\"delta\":{\"content\":\"{broken\"}}]}\n\ndata: {\"choices\":[{\"finish_reason\":\"stop\"}]}\n\n").unwrap();
 assert!(repair::recoverable_output(&ielts_core::cloud::error("模型输出不是完整 JSON"),&f).unwrap().is_some());
 assert!(repair::recoverable_output(&ielts_core::cloud::error("模型输出不完整或达到 Token 上限"),&f).unwrap().is_some());
}

#[tokio::test]
async fn failed_repair_attempt_does_not_issue_another_request_on_continue() {
 let tmp=tempfile::tempdir().unwrap();let path=tmp.path().join("part.json");let original=json!({"answer":null});
 std::fs::write(path.with_extension("repair-attempt.json"),serde_json::to_vec(&json!({"originalHash":ielts_core::papers::hash(&serde_json::to_vec(&original).unwrap()),"state":"failed","error":"修复 JSON 损坏"})).unwrap()).unwrap();
 let provider=ielts_core::configuration::ProviderConfig::default();
 let e=repair::checked(&provider,"unused","system",json!("scope"),false,&path,original,|_|Err(ielts_core::cloud::error("answer 缺失"))).await.unwrap_err();
 assert!(e.message.contains("已反馈"));assert!(!path.with_extension("repair-response.txt").exists());
}

#[test]
fn source_correction_request_is_reserved_before_network_send() {
 let tmp=tempfile::tempdir().unwrap();let path=tmp.path().join("unit.repair.json");
 repair::reserve_source_correction(&path).unwrap();
 assert!(repair::reserve_source_correction(&path).unwrap_err().message.contains("已反馈"));
}
