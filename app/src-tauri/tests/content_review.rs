use ielts_core::cloud::review;
use serde_json::json;
#[test]
fn matching_group_repairs_include_contiguous_items_with_the_same_shared_instructions(){
 let mut qs:Vec<_>=(1..=5).map(|n|json!({"id":format!("q{n}"),"kind":"matching","points":1,"prompt":format!("Match each role. Choose A–E. Role {n}:"),"slots":[format!("Role {n}")]})).collect();qs.push(json!({"id":"q6","kind":"matching","points":1,"prompt":"A different group's instructions","slots":["Other"]}));let m=json!({"questions":qs});
 assert_eq!(review::question_repair_positions(&m,&[json!({"questionId":"listening-2-2"})],"listening",2).unwrap(),vec![0,1,2,3,4]);
}
#[test]
fn omitted_adjudication_decisions_report_exact_question_ids_for_bounded_feedback(){
 let proposed=json!({"issues":[{"questionId":"listening-3-5"},{"questionId":"listening-3-9"}]});let checked=json!({"decisions":[{"questionId":"listening-3-5","disposition":"withdraw","reason":"This group correctly covers two numbered positions."}]});
 let error=review::adjudication_verdict(&proposed,&checked).unwrap_err();assert!(error.message.contains("listening-3-9"));
}
#[test]
fn grouped_review_ids_resolve_to_all_original_questions_without_allowing_unknown_ids(){
 let m=json!({"questions":[{"id":"q1"},{"id":"q2"},{"id":"q3"}]});
 let issues=vec![json!({"questionId":"reading-1-1","affectedQuestionIds":["reading-1-1","reading-1-2"]})];
 assert_eq!(review::question_repair_positions(&m,&issues,"reading",1).unwrap(),vec![0,1]);
 let mut patch=json!({"repairs":[{"id":"reading-1-1","prompt":"fixed"},{"id":"q2","prompt":"fixed"}]});review::normalize_repair_ids(&m,&mut patch,"reading",1).unwrap();assert_eq!(patch["repairs"][0]["id"],"q1");
 assert!(review::merge_question_repairs(&m,&patch,&[0,1]).is_ok());
 assert!(review::question_repair_positions(&m,&[json!({"questionId":"reading-2-1"})],"reading",1).is_err());
 let colliding=json!({"questions":[{"id":"reading-1-2"},{"id":"reading-1-1"}]});assert_eq!(review::question_repair_positions(&colliding,&[json!({"questionId":"reading-1-1"})],"reading",1).unwrap(),vec![0]);
}
#[test]
fn review_context_keeps_one_source_copy_and_verifiable_evidence_references(){
 let text="The immutable source says that wetlands remove nutrients.";let c=json!({"paper":{"sections":[{"section":"reading","paragraphs":[{"id":"reading-1-p1","part":1,"text":text}],"questions":[{"id":"reading-1-1","part":1,"evidence":[text],"answer":{"accepted":["wetlands"]}}]}]},"sourceMaterials":[{"section":"reading","part":1,"material":{"paragraphs":[{"id":"p1","text":text}],"questions":[{"evidence":[text]}],"visual":{"title":"Preserved chart"},"transcript":[]}}]});
 let small=review::compact_context(&c);assert_eq!(small["paper"]["sections"][0]["questions"][0]["evidenceRefs"],json!(["reading-1-p1"]));assert_eq!(small.to_string().matches(text).count(),1);assert_eq!(small["sourceMaterials"][0]["material"]["visual"]["title"],"Preserved chart");assert_eq!(small["paper"]["sections"][0]["questions"][0]["answer"],c["paper"]["sections"][0]["questions"][0]["answer"]);assert!(!review::blind_context(&small).to_string().contains("evidenceRefs"));
}
#[test]
fn explicit_issue_decisions_determine_verdict_instead_of_a_model_pass_flag(){
 let p=json!({"issues":[{"questionId":"reading-1-9","section":"reading","part":1,"reason":"missing comparison"}]});
 let v=review::adjudication_verdict(&p,&json!({"pass":false,"decisions":[{"questionId":"reading-1-9","disposition":"withdraw","reason":"The missing comparison makes NOT GIVEN correct; there is no defect."}]})).unwrap();assert_eq!(v["pass"],true);assert!(v["issues"].as_array().unwrap().is_empty());
 assert!(review::adjudication_verdict(&p,&json!({"decisions":[]})).is_err());assert!(review::adjudication_verdict(&p,&json!({"decisions":[{"questionId":"reading-1-9","disposition":"uncertain"}]})).is_err());
}
#[test]
fn extra_nonblocking_notes_do_not_reject_a_valid_adjudication(){
 let proposed=json!({"issues":[{"questionId":"reading-1-1","reason":"style"}]});let checked=json!({"pass":true,"issues":[],"advisories":[{"questionId":"reading-1-1","reason":"This is an optional editorial preference, with no demonstrated defect."},{"questionId":"reading-1-2","reason":"Another valid item."}]});review::validate_adjudication(&proposed,&checked).unwrap();
}
#[tokio::test]
async fn saved_raw_review_response_survives_a_crash_before_parsed_checkpoint(){
 let t=tempfile::tempdir().unwrap();let raw=t.path().join("raw.response.txt");let data=json!({"paper":"v1"});let first=review::checkpointed(t.path(),"audit",&data,||async{std::fs::write(&raw,b"saved").unwrap();Err(ielts_core::cloud::error("simulated stop after response"))}).await;assert!(first.is_err());let calls=std::sync::atomic::AtomicUsize::new(0);let v=review::checkpointed_with_response(t.path(),"audit",&data,Some(&raw),||async{ielts_core::cloud::request::recorded(&raw,&json!({}),||async{calls.fetch_add(1,std::sync::atomic::Ordering::SeqCst);Ok(b"new".to_vec())}).await?;Ok(json!({"pass":true,"issues":[]}))}).await.unwrap();assert_eq!(v["pass"],true);assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst),0);
}
#[test]
fn question_repairs_freeze_material_numbering_and_unaffected_questions(){
 let original=json!({"paragraphs":[{"id":"p1","text":"An immutable source."}],"questions":[{"id":"q1","number":1,"part":1,"points":1,"kind":"gap_fill","prompt":"Original one"},{"id":"q2","number":2,"part":1,"points":1,"kind":"gap_fill","prompt":"Original two"}]});let fixed=ielts_core::cloud::review::merge_question_repairs(&original,&json!({"repairs":[{"id":"q2","prompt":"Corrected question","evidenceSourceIds":["p1"]}]}),&[1]).unwrap();assert_eq!(fixed["paragraphs"],original["paragraphs"]);assert_eq!(fixed["questions"][0],original["questions"][0]);assert_eq!(fixed["questions"][1]["number"],2);assert_eq!(fixed["questions"][1]["prompt"],"Corrected question");assert!(ielts_core::cloud::review::merge_question_repairs(&original,&json!({"repairs":[{"id":"q2","points":10}]}),&[1]).is_err());
}
#[test]
fn adjudication_must_account_for_every_proposed_defect_and_prove_retained_blockers(){
 let proposed=json!({"pass":false,"issues":[{"questionId":"reading-1-1","reason":"duplicate theme"}]});let checked=json!({"pass":true,"issues":[],"advisories":[{"questionId":"reading-1-1","reason":"主题相同不妨碍两个问题分别存在明确答案，仅为可选设计建议"}]});ielts_core::cloud::review::validate_adjudication(&proposed,&checked).unwrap();assert!(ielts_core::cloud::review::validate_adjudication(&proposed,&json!({"pass":true,"issues":[]})).is_err());assert!(ielts_core::cloud::review::validate_adjudication(&proposed,&json!({"pass":false,"issues":[{"questionId":"reading-2-9","reason":"new invented issue","proof":"This was never one of the proposed issues."}]})).is_err());
}
use std::sync::atomic::{AtomicUsize,Ordering};
#[tokio::test]
async fn text_review_checkpoint_reuses_success_and_never_repeats_uncertain_paid_requests(){
 let tmp=tempfile::tempdir().unwrap();let calls=AtomicUsize::new(0);
 let v=review::checkpointed(tmp.path(),"audit",&json!({"paper":"v1"}),||async{calls.fetch_add(1,Ordering::SeqCst);Ok(json!({"pass":true,"issues":[]}))}).await.unwrap();
 let again=review::checkpointed(tmp.path(),"audit",&json!({"paper":"v1"}),||async{panic!("must reuse cache")}).await.unwrap();assert_eq!(v,again);assert_eq!(calls.load(Ordering::SeqCst),1);
 let _=review::checkpointed(tmp.path(),"audit",&json!({"paper":"v2"}),||async{calls.fetch_add(1,Ordering::SeqCst);Err(ielts_core::cloud::error("network timeout"))}).await;
 assert!(review::checkpointed(tmp.path(),"audit",&json!({"paper":"v2"}),||async{panic!("uncertain request must not repeat")}).await.is_err());
 assert_eq!(calls.load(Ordering::SeqCst),2);
}
#[test]
fn audit_cannot_be_both_passed_and_blocking_and_targets_must_be_locatable(){
 assert!(review::validate_verdict(&json!({"pass":true,"issues":[{"questionId":"q1","reason":"wrong answer"}]})).is_err());
 let plan=vec![("writing".to_string(),1),("writing".to_string(),2)];
 assert_eq!(review::targets(&json!({"issues":[{"questionId":"writing-1-1","reason":"missing units"}]}),&plan).unwrap(),vec![0]);
 assert!(review::targets(&json!({"issues":[{"questionId":"unknown","reason":"maybe"}]}),&plan).is_err());
}
