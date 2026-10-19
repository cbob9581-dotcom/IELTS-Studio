use ielts_core::source_library::{question_check,matching};
use serde_json::json;
#[test]
fn malformed_choice_groups_and_unanswerable_matching_are_rejected(){
 assert!(question_check::validate(&json!({"section":"reading","questions":[{"number":23,"kind":"multiple_choice","prompt":"Which TWO advantages?","points":1,"options":["A","B","C"]}]})).is_err());
 assert!(question_check::validate(&json!({"section":"reading","questions":[{"number":14,"kind":"matching","prompt":"match","points":1}]})).is_err());
 assert!(question_check::validate(&json!({"section":"reading","questions":[{"number":23,"kind":"multiple_choice","prompt":"Which TWO?","points":2,"options":["A","B","C"]}]})).is_ok());
}
#[test]
fn source_letters_are_mapped_to_actual_widget_values_without_changing_source(){
 let e=json!({"kind":"answers","answer":{"accepted":[["C","D"]],"caseSensitive":false}});
 let mut q=json!({"kind":"multiple_choice","options":["one","two","three","four"],"points":2});matching::attach(&mut q,&e,false).unwrap();assert_eq!(q["answer"]["accepted"],json!([["three","four"]]));assert_eq!(e["answer"]["accepted"],json!([["C","D"]]));
 let mut q=json!({"kind":"matching","slots":["14"],"options":["A","B"]});matching::attach(&mut q,&json!({"kind":"answers","answer":{"accepted":["A"],"caseSensitive":false}}),false).unwrap();assert_eq!(q["answer"]["accepted"],json!([{"14":"A"}]));
 let mut q=json!({"kind":"matching","slots":["14"],"options":["A. paragraph A","B. paragraph B"]});matching::attach(&mut q,&json!({"kind":"answers","answer":{"accepted":[{"14":"A"}],"caseSensitive":false}}),false).unwrap();assert_eq!(q["answer"]["accepted"],json!([{"14":"A. paragraph A"}]));
}
#[test]
fn html_entities_preserve_exact_quote_meaning_without_accepting_changed_answers(){
 let b=json!([{"id":"b1","text":"<td>City &amp; South London: FALSE &#38; not TRUE</td>"}]);
 assert!(matching::verify_entry(&json!({"blockIds":["b1"],"quote":"City & South London: FALSE & not TRUE"}),&b).is_ok());
 assert!(matching::verify_entry(&json!({"blockIds":["b1"],"quote":"City & South London: TRUE"}),&b).is_err());
}
