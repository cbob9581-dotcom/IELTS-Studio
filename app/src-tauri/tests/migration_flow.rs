use ielts_core::{
    attempts, migration, papers,
    storage::{StorageRoots, Store},
};
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
