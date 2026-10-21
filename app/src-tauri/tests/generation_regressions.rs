use ielts_core::cloud::generation;
use serde_json::{json,Value};
#[test]
fn mapping_answer_object_is_wrapped_without_changing_its_slots_or_values(){
 let mut v=duplicate_group();v["questions"][0]=json!({"id":"q1","number":1,"part":1,"kind":"matching","points":1,"prompt":"Which paragraph contains this information?","slots":["statement"],"options":["A","B"],"answer":{"accepted":{"statement":"B"},"caseSensitive":false},"evidence":["oak and pine"]});
 generation::normalize_generated(&mut v,"reading",1).unwrap();generation::validate_part(&v,"reading",1,None).unwrap();assert_eq!(v["questions"][0]["answer"]["accepted"],json!([{"statement":"B"}]));
}
#[test]
fn generated_word_limit_reconciliation_changes_visible_instructions_without_changing_source_or_answers(){
 let mut v=json!({"paragraphs":[{"id":"p1","part":1,"text":"The source says drought tolerant plants."}],"questions":[{"id":"q1","kind":"gap_fill","prompt":"Write NO MORE THAN TWO WORDS. Choose ____ for dry roofs.","answer":{"accepted":["drought tolerant plants"],"maxWords":2}}]});
 let source=v["paragraphs"].clone();let answer=v["questions"][0]["answer"]["accepted"].clone();generation::reconcile_generated_limits(&mut v).unwrap();assert_eq!(v["paragraphs"],source);assert_eq!(v["questions"][0]["answer"]["accepted"],answer);assert!(v["questions"][0]["prompt"].as_str().unwrap().contains("THREE WORDS"));assert_eq!(v["questions"][0]["answer"]["maxWords"],3);
 let mut too_long=v.clone();too_long["questions"][0]["answer"]["accepted"]=json!(["a very long invented response"]);assert!(generation::reconcile_generated_limits(&mut too_long).is_err());
}
fn duplicate_group()->Value {
 let group=json!({"id":"q9","number":9,"part":1,"kind":"multiple_choice","prompt":"Questions 9 and 10: choose TWO letters.","points":2,"options":["A. oak","B. pine","C. elm"],"answer":{"accepted":["A. oak","B. pine"],"caseSensitive":false},"evidence":["oak and pine"]});
 let mut qs:Vec<Value>=(1..=8).chain(11..=13).map(|n|json!({"id":format!("q{n}"),"number":n,"part":1,"kind":"gap_fill","points":1,"prompt":"Write ONE WORD","answer":{"accepted":["oak"],"caseSensitive":false,"maxWords":1},"evidence":["oak and pine"]})).collect();
 qs.insert(8,group.clone());let mut duplicate=group;duplicate["id"]=json!("q10");duplicate["number"]=json!(10);qs.insert(9,duplicate);
 json!({"questions":qs,"paragraphs":[{"id":"p1","part":1,"text":"oak and pine ".repeat(230)}],"transcript":[]})
}
#[test]
fn identical_numbered_copies_of_one_choose_two_group_are_normalized_without_new_answers(){
 let mut v=duplicate_group();generation::normalize_metadata(&mut v,"reading",1).unwrap();
 assert_eq!(v["questions"].as_array().unwrap().len(),12);
 assert_eq!(v["questions"][8]["answer"]["accepted"],json!([["A. oak","B. pine"]]));
 generation::validate_part(&v,"reading",1,None).unwrap();
}
#[test]
fn normalization_never_merges_different_questions_that_overlap(){
 let mut v=duplicate_group();v["questions"][9]["prompt"]=json!("A different question");
 generation::normalize_metadata(&mut v,"reading",1).unwrap();
 assert_eq!(v["questions"].as_array().unwrap().len(),13);
 let e=generation::validate_part(&v,"reading",1,None).unwrap_err();
 assert!(e.message.contains("15")&&e.message.contains("13"),"{}",e.message);
}
#[test]
fn a_complete_shorter_listening_dialogue_is_not_rejected_by_an_invented_official_word_limit(){
 let text="We can plant oak and pine trees in the new park next month. ";
 let ts:Vec<_>=[2,0,1,0,1,0,1,0,1].iter().map(|n|json!({"speaker":n,"text":text.repeat(3)})).collect();
 let qs:Vec<_>=(1..=10).map(|n|json!({"id":format!("q{n}"),"number":n,"part":1,"kind":"gap_fill","points":1,"prompt":"Write ONE WORD","evidence":["oak and pine"],"answer":{"accepted":["oak"],"caseSensitive":false,"maxWords":1}})).collect();
 generation::validate_part(&json!({"questions":qs,"paragraphs":[],"transcript":ts}),"listening",1,None).unwrap();
}
#[test]
fn prompts_give_an_explicit_scoring_plan_and_separate_subjective_tasks(){
 let p=generation::prompt("reading",1,Some("multiple_choice"),"").unwrap();
 assert!(p.contains("questionPlan")&&p.contains("coveredNumbers"));
 let p=generation::prompt("writing",1,None,"").unwrap();
 assert!(!p.contains("exactly 1 numbered scored answer"));
 assert!(p.contains("units"));
}
#[test]
fn distinct_question_numbers_cannot_reuse_the_same_score_position(){
 let mut v=duplicate_group();generation::normalize_metadata(&mut v,"reading",1).unwrap();
 v["questions"][1]["number"]=json!(1);
 assert!(generation::validate_part(&v,"reading",1,None).is_err());
}
#[test]
fn explicit_word_limit_is_read_from_the_task_but_incompatible_answers_are_rejected(){
 let mut v=duplicate_group();v["questions"][0]["prompt"]=json!("Write NO MORE THAN TWO WORDS AND/OR A NUMBER.");v["questions"][0]["answer"].as_object_mut().unwrap().remove("maxWords");
 generation::normalize_metadata(&mut v,"reading",1).unwrap();assert_eq!(v["questions"][0]["answer"]["maxWords"],2);
 v["questions"][0]["answer"]["accepted"]=json!(["three separate words"]);
 let e=generation::validate_part(&v,"reading",1,None).unwrap_err();assert!(e.message.contains("字数")&&e.message.contains("q1"));
}
#[test]
fn flattened_answer_fields_and_single_evidence_strings_are_only_repacked_without_inventing_content(){
 let mut v=duplicate_group();let q=&mut v["questions"][0];let answer=q.as_object_mut().unwrap().remove("answer").unwrap();
 for (key,value) in answer.as_object().unwrap(){q[key]=value.clone();}q["evidence"]=json!("oak and pine");
 generation::normalize_metadata(&mut v,"reading",1).unwrap();
 assert_eq!(v["questions"][0]["answer"]["accepted"],json!(["oak"]));assert!(v["questions"][0].get("accepted").is_none());assert_eq!(v["questions"][0]["evidence"],json!(["oak and pine"]));
 generation::validate_part(&v,"reading",1,None).unwrap();
}
#[test]
fn schema_errors_are_found_in_the_text_step_before_audio_or_full_paper_assembly(){
 let mut v=duplicate_group();generation::normalize_metadata(&mut v,"reading",1).unwrap();v["questions"][0]["unsupportedField"]=json!("not in contract");
 assert!(generation::validate_part(&v,"reading",1,None).is_err());
}
#[test]
fn source_ids_bind_actual_source_text_instead_of_model_retyped_quotations(){
 let mut v=duplicate_group();v["questions"][0]["evidenceSourceIds"]=json!(["p1"]);v["questions"][0]["evidence"]=json!(["invented quotation"]);
 generation::normalize_metadata(&mut v,"reading",1).unwrap();
 assert_eq!(v["questions"][0]["evidence"][0],v["paragraphs"][0]["text"]);assert!(v["questions"][0].get("evidenceSourceIds").is_none());
 let mut bad=duplicate_group();bad["questions"][0]["evidenceSourceIds"]=json!(["p999"]);
 assert!(generation::normalize_metadata(&mut bad,"reading",1).is_err());
}
#[test]
#[ignore = "uses saved local responses; no cloud requests"]
fn replay_saved_failed_outputs_without_new_model_or_tts_requests(){
 let root=std::path::PathBuf::from(std::env::var("IELTS_FAILURE_FIXTURES").unwrap());
 for (id,section) in [("02368b8c-7530-405a-ad5a-eb3ea77d703d","listening"),("506dcdf3-c250-4d52-b806-6b26ac4aa432","listening"),("5eb9b05d-e0b7-424d-97fe-fd9a8bcac746","reading")]{
  let mut v:Value=serde_json::from_slice(&std::fs::read(root.join(id).join("part-0-raw.json")).unwrap()).unwrap();
  generation::normalize_metadata(&mut v,section,1).unwrap();
  let result=generation::validate_part(&v,section,1,None);
  if id=="02368b8c-7530-405a-ad5a-eb3ea77d703d" {let error=result.unwrap_err().message;assert!(error.contains("q6")&&error.contains("q8")&&error.contains("q10"));}
  else if id=="506dcdf3-c250-4d52-b806-6b26ac4aa432" {assert!(result.unwrap_err().message.contains("字数"));}
  else {result.unwrap_or_else(|e|panic!("{id}: {}",e.message));}
 }
}
