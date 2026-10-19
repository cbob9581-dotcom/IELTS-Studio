use ielts_core::source_library::{self as lib, layout, matching};
use serde_json::json;

#[test]
fn project_revision_and_duplicate_documents() {
    let tmp = tempfile::tempdir().unwrap();
    let mut p = lib::create(tmp.path(), "剑雅 Academic").unwrap();
    let f = tmp.path().join("book.txt"); std::fs::write(&f, "Test 1 Reading").unwrap();
    lib::add_document(tmp.path(), &mut p, &f, json!(["questions"])).unwrap();
    lib::add_document(tmp.path(), &mut p, &f, json!(["answers"])).unwrap();
    assert_eq!(p["documents"].as_array().unwrap().len(), 1);
    assert_eq!(p["documents"][0]["roles"], json!(["questions", "answers"]));
    lib::save(tmp.path(), &mut p, 0).unwrap();
    assert!(lib::save(tmp.path(), &mut p, 0).unwrap_err().message.contains("更新"));
    assert!(lib::dir(tmp.path(), "../bad").is_err());
}

#[test]
fn preserves_148_physical_pages_and_image_geometry() {
    let pages: Vec<_> = (0..148).map(|n| json!({"type":"text","text":format!("page {n}"),"page_idx":n,"bbox":[10,20,800,900]})).collect();
    let blocks = layout::normalize(&json!(pages)).unwrap();
    assert_eq!(blocks.len(), 148);
    assert_eq!(blocks[147]["page"], 148);
    assert_eq!(blocks[0]["bbox"], json!([0.01,0.02,0.8,0.9]));
    let modern = layout::normalize(&json!({"pages":[{"page_idx":8,"blocks":[{"type":"text","content":"A"},{"type":"image","image_path":"images/a.png"}]}]})).unwrap();
    assert_eq!(modern[1]["page"], 9);
    assert_eq!(modern[1]["imagePath"], "images/a.png");
}

#[test]
fn never_matches_number_without_test_identity() {
    let questions=json!([{"id":"a","test":"Test 1","section":"reading","number":1},{"id":"b","test":"Test 2","section":"reading","number":1}]);
    assert!(matching::candidates(&questions, &json!({"section":"reading","numbers":[1]})).is_empty());
    assert_eq!(matching::candidates(&questions,&json!({"test":"Test 2","section":"reading","numbers":[1]})),vec!["b"]);
}

#[test]
fn conflicting_answer_never_silently_overrides_confirmed_answer() {
    let mut q=json!({"id":"q1","answer":{"accepted":["A"],"caseSensitive":false},"answerConfirmed":true});
    matching::attach(&mut q,&json!({"id":"entry","kind":"answers","answer":{"accepted":["B"],"caseSensitive":false},"quote":"1 B","anchor":{"documentId":"d","blockIds":["b1"]}}),false).unwrap();
    assert_eq!(q["answer"]["accepted"],json!(["A"]));
    assert_eq!(q["conflicts"].as_array().unwrap().len(),1);
}

#[test]
fn source_quote_must_exist_and_model_cannot_invent_anchors() {
    let blocks=json!([{"id":"b1","page":1,"text":"1. A"}]);
    assert!(matching::verify_entry(&json!({"quote":"1. B","blockIds":["b1"]}),&blocks).is_err());
    assert!(matching::verify_entry(&json!({"quote":"1. A","blockIds":["b2"]}),&blocks).is_err());
    assert!(matching::verify_entry(&json!({"quote":"1. A","blockIds":["b1"]}),&blocks).is_ok());
}

#[test]
fn unit_boundaries_are_bounded_and_require_identity() {
    let b=vec![json!({"id":"b1","text":"one"}),json!({"id":"b2","text":"two"})];
    let spec=json!({"start":0,"end":1,"test":"Test 1","section":"reading","part":1,"role":"questions"});
    assert!(lib::make_unit(&json!({"id":"project"}),"doc",&spec,&b).is_ok());
    let mut bad=spec.clone();bad["end"]=json!(2);
    assert!(lib::make_unit(&json!({"id":"project"}),"doc",&bad,&b).is_err());
}

#[test]
fn publication_blocks_missing_answers_and_preserves_explanation_provenance() {
    let mut p=json!({"id":"project","title":"Book","units":[{"id":"unit","test":"Test 1","section":"reading","part":1,"role":"questions","state":"complete","confirmed":true,"result":{"section":{"section":"reading","questions":[{"id":"q1","number":1,"part":1,"kind":"gap_fill","points":1,"prompt":"A ___","evidence":[]}],"paragraphs":[]}}}],"documents":[],"entries":[]});
    assert!(lib::publication::assemble(&p,"Test 1",&["unit".into()],1).is_err());
    let q=&mut p["units"][0]["result"]["section"]["questions"][0];
    matching::attach(q,&json!({"id":"a","kind":"answers","answer":{"accepted":["tree"],"caseSensitive":false},"quote":"1 tree","anchor":{"documentId":"answers","blockIds":["b1"],"page":9}}),false).unwrap();
    matching::attach(q,&json!({"id":"e","kind":"explanations","text":"定位第二段","quote":"定位第二段","anchor":{"documentId":"explanations","blockIds":["b4"],"page":2}}),false).unwrap();
    let paper=lib::publication::assemble(&p,"Test 1",&["unit".into()],1).unwrap();
    assert_eq!(paper["sections"][0]["questions"][0]["reviewMaterials"][0]["anchor"]["documentId"],"explanations");
    assert_eq!(paper["sections"][0]["questions"][0]["answer"]["accepted"],json!(["tree"]));
    assert_eq!(p["units"][0]["result"]["section"]["questions"][0]["evidence"],json!([]));
}

#[test]
fn publication_rejects_mixing_tests_and_unconfirmed_ranges() {
    let p=json!({"id":"p","units":[{"id":"u","test":"Test 2","role":"questions","state":"complete","confirmed":true}],"documents":[]});
    assert!(lib::publication::assemble(&p,"Test 1",&["u".into()],1).is_err());
}

#[test]
fn rematching_drops_removed_source_answers_and_keeps_rejected_entries_rejected() {
    let mut p=json!({"units":[{"test":"Test 1","section":"reading","role":"questions","result":{"section":{"questions":[{"id":"q","number":1,"part":1}]}}}],"entries":[{"id":"a","kind":"answers","test":"Test 1","section":"reading","numbers":[1],"answer":{"accepted":["A"],"caseSensitive":false},"quote":"1 A"}]});
    matching::reconcile(&mut p).unwrap();assert!(p["units"][0]["result"]["section"]["questions"][0]["answer"].is_object());
    p["entries"][0]["state"]=json!("ignored");matching::reconcile(&mut p).unwrap();
    assert!(p["units"][0]["result"]["section"]["questions"][0].get("answer").is_none());
}

#[test]
fn writing_visual_input_keeps_all_original_images() {
    assert_eq!(ielts_core::cloud::assessment::image_ids(&json!({"imageAssetId":"a","imageAssetIds":["a","b"]})),vec!["a","b"]);
}

#[test]
fn failed_unit_commits_no_partial_answer_entries() {
    let mut p=json!({"documents":[],"units":[{"id":"u","state":"pending"}],"entries":[]});
    let mut partial=p.clone();partial["entries"]=json!([{"id":"unvalidated-answer"}]);
    lib::finish_step(&mut p,partial,"u",Some(lib::err("后续引文无效"))).unwrap();
    assert_eq!(p["entries"],json!([]));assert_eq!(p["units"][0]["state"],"failed");
}
