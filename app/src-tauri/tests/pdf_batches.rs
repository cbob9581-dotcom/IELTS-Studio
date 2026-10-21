use ielts_core::source_library::pdf_batch;
use serde_json::json;
#[test]
fn balanced_batches_cover_248_pages_without_overlap_or_missing_pages(){
 let ranges=pdf_batch::ranges(248,141988476,180,90*1024*1024).unwrap();
 assert_eq!(ranges,vec![(1,124),(125,248)]);
 assert!(pdf_batch::ranges(0,0,180,90*1024*1024).is_err());
}
#[test]
fn production_batches_keep_scanned_originals_below_zip_entry_limits(){
 assert!(pdf_batch::MAX_BYTES<=40*1024*1024);
 let plan=pdf_batch::ranges(248,141988476,pdf_batch::MAX_PAGES,pdf_batch::MAX_BYTES).unwrap();
 assert_eq!(plan,vec![(1,62),(63,124),(125,186),(187,248)]);
}
#[test]
fn merging_resets_local_ids_and_restores_original_physical_pages(){
 let batches=vec![(1,vec![json!({"id":"b1","page":1,"text":"first"})]),(125,vec![json!({"id":"b1","page":1,"text":"second"})])];
 let merged=pdf_batch::merge(batches).unwrap();assert_eq!(merged[1]["page"],125);assert_eq!(merged[1]["id"],"b2");
}
#[test]
fn cover_and_preface_are_not_forced_into_a_test_and_uppercase_test_is_normalized(){
 assert!(ielts_core::source_library::layout::scope(&json!({"test":null,"section":null,"role":"explanations"})).is_none());
 let s=ielts_core::source_library::layout::scope(&json!({"test":"TEST 1","section":"reading","role":"questions","part":1})).unwrap();
 assert_eq!(s["test"],"Test 1");
}
#[test]
fn small_unidentified_images_are_preserved_instead_of_guessed_to_be_markers(){
 let blocks=ielts_core::source_library::layout::normalize(&json!([
 {"type":"image","img_path":"qr.jpg","bbox":[121,190,181,240],"page_idx":119},
 {"type":"image","img_path":"map.jpg","bbox":[250,390,757,614],"page_idx":27}
 ])).unwrap();
 assert_eq!(blocks[0]["auxiliaryImage"],false);assert_eq!(blocks[1]["auxiliaryImage"],false);
}
#[test]
fn readable_html_tables_do_not_require_a_vision_call(){
 assert!(!ielts_core::source_library::layout::requires_visual(&json!({"type":"table","text":"<table><tr><td>answer</td></tr></table>","imageAssetId":"table"})));
 assert!(ielts_core::source_library::layout::requires_visual(&json!({"type":"image","text":"","imageAssetId":"small-legend","bbox":[0.1,0.1,0.15,0.15]})));
}
#[test]
fn bounded_batch_selection_skips_successes_and_failed_batches_do_not_discard_siblings(){
 let mut document=json!({"parseSegments":[{"firstPage":1,"lastPage":10,"state":"complete"},{"firstPage":11,"lastPage":20,"state":"pending"},{"firstPage":21,"lastPage":30,"state":"pending"},{"firstPage":31,"lastPage":40,"state":"pending"}]});
 assert_eq!(pdf_batch::pending_indices(&document,2).unwrap(),vec![1,2]);
 pdf_batch::record_failure(&mut document,1,"timeout").unwrap();assert_eq!(document["parseSegments"][0]["state"],"complete");assert_eq!(pdf_batch::pending_indices(&document,2).unwrap(),vec![2,3]);
}
#[test]
#[ignore = "Requires the user-authorized Cambridge 17 PDF; run with IELTS_PDF_FIXTURE"]
fn actual_scanned_pdf_can_be_split_and_reopened_without_changing_original(){
 let source=std::path::PathBuf::from(std::env::var("IELTS_PDF_FIXTURE").unwrap());
 let tmp=tempfile::tempdir().unwrap();
 let result=pdf_batch::prepare(&source,tmp.path(),180,90*1024*1024).unwrap();
 assert_eq!(result["pageCount"],248);let parts=result["segments"].as_array().unwrap();assert_eq!(parts.len(),2);
 for p in parts{let d=lopdf::Document::load(p["path"].as_str().unwrap()).unwrap();assert!(d.get_pages().len()<=180);}
}
