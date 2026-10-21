use ielts_core::{
    attempts, migration, papers,
    storage::{StorageRoots, Store},
};
#[test]
fn asset_migration_snapshots_live_task_database_and_omits_runtime_locks(){
 let t=tempfile::tempdir().unwrap();let roots=StorageRoots{assets:t.path().join("a").to_string_lossy().into(),study:t.path().join("b").to_string_lossy().into()};let s=Store::open(roots.clone()).unwrap();
 let mut q=ielts_core::task_queue::Queue::open(std::path::Path::new(&roots.assets)).unwrap();let task=q.enqueue(ielts_core::task_queue::TaskRequest{kind:"source".into(),owner:"book".into(),targets:vec!["one".into(),"two".into()],title:"Source import".into()}).unwrap();q.claim().unwrap();q.complete_target(&task.id,"one").unwrap();q.reserve_request(&task.id,"attempt-one","test-model",4).unwrap();
 let lock=std::fs::File::create(std::path::Path::new(&roots.assets).join(".task-engine.lock")).unwrap();lock.try_lock().unwrap();
 let target=StorageRoots{assets:t.path().join("new-a").to_string_lossy().into(),study:roots.study};migration::copy_verified(&s,target.clone()).unwrap();
 assert!(!std::path::Path::new(&target.assets).join(".task-engine.lock").exists());let migrated=ielts_core::task_queue::Queue::open(std::path::Path::new(&target.assets)).unwrap();let moved=migrated.read(&task.id).unwrap();assert_eq!(moved.completed,vec!["one"]);assert_eq!(moved.usage.requests,1);assert_eq!(q.read(&task.id).unwrap().completed,vec!["one"]);
}
#[test]
fn allows_changing_only_the_study_directory() {
    let t = tempfile::tempdir().unwrap();
    let a = StorageRoots {
        assets: t.path().join("a").to_string_lossy().into(),
        study: t.path().join("s").to_string_lossy().into(),
    };
    let s = Store::open(a.clone()).unwrap();
    let target = StorageRoots {
        assets: a.assets.clone(),
        study: t.path().join("new-study").to_string_lossy().into(),
    };
    assert!(migration::copy_verified(&s, target).is_ok());
}
#[test]
fn migration_preserves_original_and_reopens_with_same_data() {
    let t = tempfile::tempdir().unwrap();
    let a = StorageRoots {
        assets: t.path().join("原 资产").to_string_lossy().into(),
        study: t.path().join("原 记录").to_string_lossy().into(),
    };
    let s = Store::open(a.clone()).unwrap();
    let p = serde_json::from_str(include_str!("../../test/fixtures/dev-reading.json")).unwrap();
    papers::install(&s, &p, t.path()).unwrap();
    let target = StorageRoots {
        assets: t.path().join("新 资产").to_string_lossy().into(),
        study: t.path().join("新 记录").to_string_lossy().into(),
    };
    let moved = migration::copy_verified(&s, target.clone()).unwrap();
    assert_eq!(papers::load(&moved, "dev-reading", 1).unwrap(), p);
    assert!(std::path::Path::new(&a.study)
        .join("study.sqlite3")
        .exists());
    assert!(attempts::list(&moved).unwrap().is_empty());
    assert_eq!(moved.roots, target);
}
#[test]
fn rejects_nested_nonempty_and_conflicting_targets() {
    let t = tempfile::tempdir().unwrap();
    let a = StorageRoots {
        assets: t.path().join("a").to_string_lossy().into(),
        study: t.path().join("s").to_string_lossy().into(),
    };
    let s = Store::open(a.clone()).unwrap();
    assert!(migration::copy_verified(&s, a.clone()).is_err());
    let nested = StorageRoots {
        assets: t.path().join("a/nested").to_string_lossy().into(),
        study: t.path().join("new").to_string_lossy().into(),
    };
    assert!(migration::copy_verified(&s, nested).is_err());
    let asset = t.path().join("occupied");
    std::fs::create_dir(&asset).unwrap();
    std::fs::write(asset.join("keep"), "user data").unwrap();
    assert!(migration::copy_verified(
        &s,
        StorageRoots {
            assets: asset.to_string_lossy().into(),
            study: t.path().join("new-study").to_string_lossy().into()
        }
    )
    .is_err());
    assert!(asset.join("keep").exists());
}
