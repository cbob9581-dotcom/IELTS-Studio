use ielts_core::{
    attempts, papers,
    storage::{StorageRoots, Store},
};
use serde_json::{json, Value};
fn paper() -> Value {
    serde_json::from_str(include_str!("../../test/fixtures/dev-reading.json")).unwrap()
}
fn store(dir: &std::path::Path) -> Store {
    Store::open(StorageRoots {
        assets: dir.join("资产 空间").to_string_lossy().to_string(),
        study: dir.join("学习 记录").to_string_lossy().to_string(),
    })
    .unwrap()
}
fn snapshot() -> Value {
    json!({"id":"attempt-1","paperId":"dev-reading","paperVersion":1,"examRunId":null,"mode":"training","section":"reading","status":"answering","responses":[{"questionId":"r1","value":"solar"}],"annotations":[],"revision":0,"clock":{"durationMs":600000,"deadlineUtcMs":null,"lastObservedUtcMs":1000,"remainingMs":600000},"interruptionEvents":[],"scope":{},"flagged":[]})
}
#[test]
fn missing_existing_database_is_reported_without_silently_replacing_it() {
    let t = tempfile::tempdir().unwrap();
    let s = store(t.path());
    let roots = s.roots.clone();
    drop(s);
    std::fs::rename(
        std::path::Path::new(&roots.study),
        t.path().join("offline-study"),
    )
    .unwrap();
    assert!(Store::reopen(roots.clone()).is_err());
    assert!(!std::path::Path::new(&roots.study).exists());
}
#[test]
fn chinese_paths_revision_and_idempotent_submission() {
    let temp = tempfile::tempdir().unwrap();
    let s = store(temp.path());
    papers::install(&s, &paper(), temp.path()).unwrap();
    let saved = attempts::save(&s, snapshot(), 0).unwrap();
    assert_eq!(saved["revision"], 1);
    assert!(attempts::save(&s, snapshot(), 0).is_err());
    drop(s);
    let s = store(temp.path());
    assert_eq!(
        attempts::load(&s, "attempt-1").unwrap()["responses"][0]["value"],
        "solar"
    );
    let submitted = attempts::submit(&s, "attempt-1", 1).unwrap();
    assert_eq!(submitted["status"], "submitted");
    assert_eq!(attempts::submit(&s, "attempt-1", 1).unwrap(), submitted);
    assert_eq!(attempts::list(&s).unwrap().len(), 1);
    assert!(attempts::save(&s, snapshot(), 2).is_err());
}
#[test]
fn rejects_bad_bundle_without_publishing() {
    let temp = tempfile::tempdir().unwrap();
    let s = store(temp.path());
    let mut p = paper();
    p["assets"] =
        json!([{"id":"a","path":"../outside.wav","sha256":"a".repeat(64),"mediaType":"audio/wav"}]);
    assert!(papers::install(&s, &p, temp.path()).is_err());
    p["assets"][0]["path"] = json!("absent.wav");
    assert!(papers::install(&s, &p, temp.path()).is_err());
    assert!(papers::list(&s).unwrap().is_empty());
    p = paper();
    p["sections"][0]["questions"][0]["kind"] = json!("unknown");
    assert!(papers::install(&s, &p, temp.path()).is_err());
    let file = temp.path().join("corrupt.json");
    std::fs::write(&file, b"not json").unwrap();
    assert!(papers::import(&s, &file).is_err());
}
#[test]
fn immutable_versions_and_write_failure_preserve_saved_state() {
    let temp = tempfile::tempdir().unwrap();
    let s = store(temp.path());
    let p = paper();
    papers::install(&s, &p, temp.path()).unwrap();
    let mut changed = p.clone();
    changed["title"] = json!("mutated");
    assert!(papers::install(&s, &changed, temp.path()).is_err());
    attempts::save(&s, snapshot(), 0).unwrap();
    s.db.execute_batch("CREATE TRIGGER fail_update BEFORE UPDATE ON attempts BEGIN SELECT RAISE(ABORT, 'disk failure'); END;").unwrap();
    let mut edit = snapshot();
    edit["responses"][0]["value"] = json!("bad");
    assert!(attempts::save(&s, edit, 1).is_err());
    assert_eq!(
        attempts::load(&s, "attempt-1").unwrap()["responses"][0]["value"],
        "solar"
    );
}
