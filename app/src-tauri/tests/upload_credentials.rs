#[test]
fn temporary_upload_location_is_protected_and_removed() {
    use ielts_core::credentials;
    let id = format!("upload-test-{}", uuid::Uuid::new_v4());
    credentials::save_upload(&id, "synthetic-local-test-only").unwrap();
    let read = credentials::read_upload(&id);
    let cleared = credentials::clear_upload(&id);
    assert_eq!(read.unwrap().as_deref(), Some("synthetic-local-test-only"));
    cleared.unwrap();
    assert!(credentials::read_upload(&id).unwrap().is_none());
}
