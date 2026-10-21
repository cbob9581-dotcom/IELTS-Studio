use ielts_core::{source_library::{self as lib,recovery},papers};
use serde_json::{json,Value};
#[test]
fn parsed_book_moves_to_source_library_without_reparsing_or_trusting_rejected_answers(){
 let source=tempfile::tempdir().unwrap();let target=tempfile::tempdir().unwrap();
 let raw=b"Original book";std::fs::write(source.path().join("source.pdf"),raw).unwrap();
 std::fs::write(source.path().join("original.md"),"Test 1\n\nREADING\n\nREADING PASSAGE 1\n\nA passage.").unwrap();
 let ext=source.path().join("extracted");std::fs::create_dir_all(&ext).unwrap();
 let blocks=json!([{"type":"text","text":"TEST 1","page_idx":0},{"type":"text","text":"READING","page_idx":0},{"type":"text","text":"READING PASSAGE 1","page_idx":0},{"type":"text","text":"A passage.","page_idx":0}]);
 std::fs::write(ext.join("content_list.json"),serde_json::to_vec(&blocks).unwrap()).unwrap();
 let job=json!({"id":"old-import","kind":"import","state":"failed","request":{"extension":"pdf","sourceName":"Book.pdf","sha256":papers::hash(raw)},"error":"missing answer quote"});
 let p=recovery::adopt_from(target.path(),source.path(),&job).unwrap();assert_eq!(p["documents"][0]["state"],"ready");assert!(p["units"].as_array().unwrap().len()>0);assert_eq!(p["entries"],json!([]));
 let again=recovery::adopt_from(target.path(),source.path(),&job).unwrap();assert_eq!(p["id"],again["id"]);assert_eq!(lib::list(target.path()).unwrap().len(),1);
 let read:Value=serde_json::from_slice(&std::fs::read(lib::dir(target.path(),p["id"].as_str().unwrap()).unwrap().join("project.json")).unwrap()).unwrap();assert_eq!(read["recoveredJobId"],"old-import");assert!(source.path().join("original.md").is_file());
}
#[test]
#[ignore="uses cached user-authorized parse files; no network"]
fn recover_real_cached_books_without_new_ocr(){
 let jobs=std::path::PathBuf::from(std::env::var("IELTS_RECOVERY_IMPORT_ROOT").unwrap());let target=std::path::PathBuf::from(std::env::var("IELTS_RECOVERY_PREVIEW_ROOT").unwrap());
 for entry in std::fs::read_dir(jobs).unwrap(){let path=entry.unwrap().path();if !path.join("original.md").is_file(){continue;}
  let job:Value=serde_json::from_slice(&std::fs::read(path.join("job.json")).unwrap()).unwrap();
  let p=recovery::adopt_from(&target,&path,&job).unwrap();assert!(p["documents"][0]["blockCount"].as_u64().unwrap()>500);assert!(p["entries"].as_array().unwrap().is_empty());
  println!("{}",json!({"source":job["request"]["sourceName"],"projectId":p["id"],"blocks":p["documents"][0]["blockCount"],"images":p["documents"][0]["assets"].as_array().unwrap().len(),"units":p["units"].as_array().unwrap().len(),"tests":p["units"].as_array().unwrap().iter().map(|u|u["test"].as_str().unwrap()).collect::<std::collections::BTreeSet<_>>(),"ocrRequests":0}));
 }
}
