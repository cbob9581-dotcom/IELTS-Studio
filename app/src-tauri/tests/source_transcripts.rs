use ielts_core::source_library::transcripts;
use serde_json::json;
#[test]
fn table_dialogue_preserves_roles_and_removes_only_question_annotations(){
 let b=json!([{"id":"b1","page":96,"text":"PART 1"},{"id":"b2","page":96,"text":"<table><tr><td>PETER:</td><td>Hello? Q1</td></tr><tr><td>JAN:</td><td>I need 15 boxes.</td><td>Q2</td></tr></table>"},{"id":"b3","page":96,"text":"96"}]);
 let d=transcripts::from_blocks(b.as_array().unwrap(),"Test 1",1).unwrap();
 assert_eq!(d["turns"].as_array().unwrap().len(),2);assert_eq!(d["turns"][0]["speaker"],"PETER");assert_eq!(d["turns"][0]["text"],"Hello?");assert_eq!(d["turns"][1]["text"],"I need 15 boxes.");assert_eq!(d["sourceRefs"][1]["page"],96);
}
#[test]
fn inline_roles_and_continuations_are_preserved(){
 let d=transcripts::parse("diana: Hello. tim: Fine.\ndiana: What about 1838?\nI mean the year.","user","Test 2",3).unwrap();
 assert_eq!(d["turns"].as_array().unwrap().len(),3);assert_eq!(d["turns"][2]["text"],"What about 1838? I mean the year.");
 assert_eq!(d["events"][1]["durationMs"],300);
}
#[test]
fn inline_roles_after_unfinished_sentence_are_not_spoken_as_dialogue(){
 let d=transcripts::parse("gemma: Hello. ed: Right – a new angle gemma: Yes, that is important ed: Exactly.","user","Test 2",3).unwrap();
 assert_eq!(d["turns"].as_array().unwrap().len(),4);assert_eq!(d["turns"][2]["speaker"],"GEMMA");assert_eq!(d["turns"][1]["text"],"Right – a new angle");
}
#[test]
fn ordinary_colons_remain_in_spoken_text(){
 let text="We have two choices: stay here or leave.";let d=transcripts::parse(text,"user","Test 1",2).unwrap();assert_eq!(d["turns"].as_array().unwrap().len(),1);assert_eq!(d["turns"][0]["speaker"],"NARRATOR");assert_eq!(d["turns"][0]["text"],text);
}
#[test]
fn monologue_and_long_text_split_without_losing_words(){
 let text="A long sentence with 32 birds. ".repeat(60);
 let d=transcripts::parse(&text,"user","Test 1",2).unwrap();
 let ts=d["turns"].as_array().unwrap();assert!(ts.len()>1);assert!(ts.iter().all(|t|t["text"].as_str().unwrap().chars().count()<=550));
 assert_eq!(ts.iter().map(|t|t["text"].as_str().unwrap()).collect::<Vec<_>>().join(" "),text.trim());assert!(transcripts::parse(" ","user","Test 1",1).is_err());
}
#[test]
#[ignore="Requires cached book"]
fn real_book_dialogue_has_complete_first_part(){
 let b:Vec<serde_json::Value>=serde_json::from_slice(&std::fs::read(std::env::var("IELTS_BLOCKS_FIXTURE").unwrap()).unwrap()).unwrap();
 let units=ielts_core::source_library::outline::derive(&b,&json!(["mixed"]));
 for u in units.iter().filter(|u|u["role"]=="transcript") {let d=transcripts::from_blocks(&b[u["start"].as_u64().unwrap() as usize..=u["end"].as_u64().unwrap() as usize],u["test"].as_str().unwrap(),u["part"].as_u64().unwrap()).unwrap();assert!(!d["turns"].as_array().unwrap().is_empty());if u["test"]=="Test 1"&&u["part"]==1 {assert_eq!(d["turns"][0]["text"],"Hello?");assert!(d["turns"].as_array().unwrap().len()>15);assert!(d["rawText"].as_str().unwrap().contains("replac"));}}
}
