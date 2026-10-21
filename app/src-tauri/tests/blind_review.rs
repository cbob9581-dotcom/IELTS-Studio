use ielts_core::cloud::review;
use serde_json::json;
#[test]
fn letter_and_full_labeled_option_are_the_same_answer_but_wrong_letter_is_not(){
 let p=json!({"sections":[{"questions":[{"id":"reading-3-1","kind":"matching","slots":["statement"],"options":["A. Paragraph A","B. Paragraph B","C. Paragraph C"],"answer":{"accepted":[{"statement":"B. Paragraph B"}]}}]}]});
 assert!(review::answer_disagreements(&p,&json!({"answers":[{"questionId":"reading-3-1","answer":"B"}]})).unwrap().is_empty());
 assert_eq!(review::answer_disagreements(&p,&json!({"answers":[{"questionId":"reading-3-1","answer":"C"}]})).unwrap().len(),1);
}
#[test]
fn scalar_single_slot_answers_are_equivalent_but_multi_slot_answers_are_not(){
 let paper=json!({"sections":[{"questions":[{"id":"reading-1-1","kind":"matching","slots":["statement"],"answer":{"accepted":[{"statement":"B"}]}},{"id":"reading-1-2","kind":"multiple_choice","options":["first option","second option"],"answer":{"accepted":[["second option"]]}}]}]});
 let solved=json!({"answers":[{"questionId":"reading-1-1","answer":"B"},{"questionId":"reading-1-2","answer":"second option"}]});assert!(review::answer_disagreements(&paper,&solved).unwrap().is_empty());
 let mut multi=paper.clone();multi["sections"][0]["questions"][0]["slots"]=json!(["statement","other"]);assert_eq!(review::answer_disagreements(&multi,&solved).unwrap().len(),1);
}
#[test]
fn independent_solver_input_hides_answer_keys_and_teacher_only_evidence(){
 let context=json!({"paper":{"sections":[{"questions":[{"id":"reading-1-1","kind":"gap_fill","prompt":"Write ONE WORD","answer":{"accepted":["river"]},"evidence":["teacher hint"]}]}]},"sourceMaterials":[{"material":{"paragraphs":[{"text":"The river flooded."}],"questions":[{"answer":{"accepted":["river"]}}],"evidenceSourceRefs":{"q1":["p1"]}}}]});
 let blind=review::blind_context(&context);let text=blind.to_string();assert!(!text.contains("accepted")&&!text.contains("teacher hint")&&!text.contains("evidenceSourceRefs"));assert!(text.contains("The river flooded."));
 let solved=json!({"answers":[{"questionId":"reading-1-1","answer":"river"}]});assert!(review::answer_disagreements(&context["paper"],&solved).unwrap().is_empty());
 let wrong=json!({"answers":[{"questionId":"reading-1-1","answer":"stream"}]});assert_eq!(review::answer_disagreements(&context["paper"],&wrong).unwrap()[0]["questionId"],"reading-1-1");
 assert!(review::answer_disagreements(&context["paper"],&json!({"answers":[]})).is_err());
}
