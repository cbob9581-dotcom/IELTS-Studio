use ielts_core::source_library::outline;
use serde_json::json;
#[test]
fn older_books_use_sections_tapescripts_and_academic_headings_without_including_general_training(){
 let blocks=json!([
 {"page":1,"text":"Practice Test 1"},{"page":1,"text":"LISTENING"},{"page":1,"text":"SECTION 1"},{"page":1,"text":"Questions 1-10"},
 {"page":2,"text":"ACADEMIC READING"},{"page":2,"text":"READING PASSAGE 1"},{"page":2,"text":"A source passage."},
 {"page":3,"text":"TAPESCRIPTS"},{"page":3,"text":"TEST 1"},{"page":3,"text":"SECTION 1"},{"page":3,"text":"MAN: Hello."},
 {"page":4,"text":"TEST A WRITING TASK 1 (GENERAL TRAINING)"},{"page":4,"text":"WRITING TASK 1"},{"page":4,"text":"Write a letter."}
 ]);
 let units=outline::derive(blocks.as_array().unwrap(),&json!(["mixed"]));
 assert!(units.iter().any(|u|u["section"]=="listening"&&u["role"]=="questions"));
 assert!(units.iter().any(|u|u["section"]=="listening"&&u["role"]=="transcript"));
 assert!(!units.iter().any(|u|u["end"].as_u64().unwrap()>=11));
}
#[test]
fn audioscripts_keep_listening_across_tests() {
 let b=json!([
 {"text":"Audioscripts","page":90},{"text":"TEST 1","page":90},
 {"text":"PART 1","page":90},{"text":"PETER: Hello?","page":90},
 {"text":"TEST 2","page":91},{"text":"PART 1","page":91},
 {"text":"JAN: Good morning.","page":91},{"text":"PART 2","page":92},
 {"text":"Guide: Welcome.","page":92}
 ]);
 let s=outline::derive(b.as_array().unwrap(),&json!(["mixed"]));
 assert_eq!(s.len(),3,"{s:?}");
 assert_eq!(s[1]["test"],"Test 2");assert_eq!(s[1]["role"],"transcript");
 assert_eq!(s[1]["start"],5);assert_eq!(s[1]["end"],6);
}
#[test]
fn repeated_audioscripts_footer_does_not_drop_continuation() {
 let b=json!([{"text":"Audioscripts","page":1},{"text":"TEST 1","page":1},{"text":"PART 3","page":1},{"text":"JAN: Start.","page":1},{"text":"Audioscripts","page":1},{"text":"PETER: Still talking.","page":2}]);
 let s=outline::derive(b.as_array().unwrap(),&json!(["mixed"]));
 assert_eq!(s.len(),1);assert_eq!(s[0]["end"],5);
}
#[test]
fn transcript_test_changes_follow_block_order_not_page_footer(){
 let b=json!([{"text":"Audioscripts","page":1},{"text":"TEST 1","page":1},{"text":"PART 4","page":1},{"text":"Beginning.","page":1},{"text":"Previous test still talking.","page":2},{"text":"TEST 2","page":2},{"text":"PART 1","page":2},{"text":"JAN: New test.","page":2}]);
 let s=outline::derive(b.as_array().unwrap(),&json!(["mixed"]));assert_eq!(s[0]["end"],4);assert_eq!(s[1]["test"],"Test 2");
}
#[test]
#[ignore="Requires cached user-authorized book"]
fn real_book_has_all_sixteen_audioscripts_and_page_continuations() {
 let b:Vec<serde_json::Value>=serde_json::from_slice(&std::fs::read(std::env::var("IELTS_BLOCKS_FIXTURE").unwrap()).unwrap()).unwrap();
 let s=outline::derive(&b,&json!(["mixed"]));
 let scripts:Vec<_>=s.iter().filter(|u|u["role"]=="transcript").collect();assert_eq!(scripts.len(),16);
 for t in 1..=4 {for p in 1..=4 {assert_eq!(scripts.iter().filter(|u|u["test"]==format!("Test {t}")&&u["part"]==p).count(),1);}}
 let p3=scripts.iter().find(|u|u["test"]=="Test 1"&&u["part"]==3).unwrap();assert!(p3["end"].as_u64().unwrap()>=1377);
 let p4=scripts.last().unwrap();assert!(p4["end"].as_u64().unwrap()>=1567);
}
#[test]
fn explicit_passage_and_task_headers_override_previous_part_and_coalesce_page_headers(){
 let b=json!([
 {"text":"Contents","page":1},{"text":"Test 1","page":1},
 {"text":"Test 1","page":2},{"text":"READING","page":2},{"text":"READING PASSAGE 1","page":2},{"text":"Article A","page":2},
 {"text":"Test 1","type":"header","page":3},{"text":"Questions 1–13","page":3},
 {"text":"READING PASSAGE 2","page":4},{"text":"Article B","page":4},
 {"text":"WRITING TASK 1","page":5},{"text":"Map task","page":5},
 {"text":"WRITING TASK 2","page":6},{"text":"Essay task","page":6}
 ]);
 let specs=outline::derive(b.as_array().unwrap(),&json!(["mixed"]));
 let r:Vec<_>=specs.iter().filter(|s|s["section"]=="reading").collect();assert_eq!(r.len(),2);assert_eq!(r[0]["part"],1);assert_eq!(r[1]["part"],2);
 assert_eq!(r[0]["start"],4);assert_eq!(r[0]["end"],7);
 let w:Vec<_>=specs.iter().filter(|s|s["section"]=="writing").collect();assert_eq!(w.len(),2);assert_eq!(w[1]["part"],2);
}
#[test]
fn numeric_and_uppercase_test_names_are_canonical(){
 let v=ielts_core::source_library::layout::scope(&json!({"test":"2","section":"reading","part":1,"role":"answers"})).unwrap();assert_eq!(v["test"],"Test 2");
}
#[test]
fn answer_keys_without_part_headings_are_retained(){
 let b=json!([{"text":"Listening and Reading answer keys","page":1},{"text":"TEST 1","page":1},{"text":"LISTENING","page":1},{"text":"1. river 2. street","page":1},{"text":"READING","page":2},{"text":"1. population 2. suburbs","page":2}]);
 let specs=outline::derive(b.as_array().unwrap(),&json!(["mixed"]));assert_eq!(specs.len(),2);assert!(specs.iter().all(|s|s["role"]=="answers"));assert_eq!(specs[1]["end"],5);
}
#[test]
#[ignore = "Uses user-authorized parsed Cambridge book; IELTS_BLOCKS_FIXTURE required"]
fn real_book_has_three_reading_passages_and_two_writing_tasks_per_test(){
 let b:Vec<serde_json::Value>=serde_json::from_slice(&std::fs::read(std::env::var("IELTS_BLOCKS_FIXTURE").unwrap()).unwrap()).unwrap();
 let specs=outline::derive(&b,&json!(["mixed"]));
 for test in ["Test 1","Test 2","Test 3","Test 4"] {
  let r:Vec<_>=specs.iter().filter(|s|s["test"]==test&&s["section"]=="reading"&&s["role"]=="questions").collect();assert_eq!(r.len(),3,"{test}: {r:?}");
  let w:Vec<_>=specs.iter().filter(|s|s["test"]==test&&s["section"]=="writing"&&s["role"]=="questions").collect();assert_eq!(w.len(),2,"{test}: {w:?}");
 }
}

#[test]
#[ignore = "Uses user-authorized parsed explanation book; IELTS_EXPLANATIONS_FIXTURE required"]
fn real_explanation_book_preserves_four_tests_without_question_units(){
 let b:Vec<serde_json::Value>=serde_json::from_slice(&std::fs::read(std::env::var("IELTS_EXPLANATIONS_FIXTURE").unwrap()).unwrap()).unwrap();
 let specs=outline::derive(&b,&json!(["explanations","essays"]));
 assert!(specs.iter().all(|s|s["role"]!="questions"));
 for test in ["Test 1","Test 2","Test 3","Test 4"] {
  let r:Vec<_>=specs.iter().filter(|s|s["test"]==test&&s["section"]=="reading").collect();assert_eq!(r.len(),3,"{test}: {r:?}");
 }
 for s in specs {let start=s["start"].as_u64().unwrap() as usize;let end=s["end"].as_u64().unwrap() as usize;let len:usize=b[start..=end].iter().map(|b|b["text"].as_str().unwrap_or("").len()).sum();assert!(len<=90000,"Oversized {s}: {len}");}
}
