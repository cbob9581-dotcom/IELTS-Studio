use ielts_core::task_queue::{Queue, TaskRequest};
use serde_json::json;
fn request(owner:&str)->TaskRequest{TaskRequest{kind:"source".into(),owner:owner.into(),targets:vec!["unit-a".into(),"unit-b".into()],title:"Private source processing".into()}}
#[test]
fn queue_is_durable_deduplicates_active_owner_and_commits_each_target(){
 let root=tempfile::tempdir().unwrap();let mut q=Queue::open(root.path()).unwrap();
 let task=q.enqueue(request("book-a")).unwrap();assert_eq!(task.state,"queued");
 let same=q.enqueue(request("book-a")).unwrap();assert_eq!(same.id,task.id);
 let running=q.claim().unwrap().unwrap();assert_eq!(running.id,task.id);assert_eq!(running.state,"running");
 assert!(q.claim().unwrap().is_none());q.complete_target(&task.id,"unit-a").unwrap();drop(q);
 let mut q=Queue::open(root.path()).unwrap();assert_eq!(q.read(&task.id).unwrap().completed,vec!["unit-a"]);
 assert_eq!(q.recover_interrupted().unwrap(),1);let paused=q.read(&task.id).unwrap();assert_eq!(paused.state,"paused");
 q.resume(&task.id).unwrap();assert_eq!(q.claim().unwrap().unwrap().completed,vec!["unit-a"]);
 q.complete_target(&task.id,"unit-b").unwrap();q.finish(&task.id,"ready",None).unwrap();
 let done=q.read(&task.id).unwrap();assert_eq!(done.completed.len(),2);assert_eq!(done.state,"ready");
 let another=q.enqueue(request("book-a")).unwrap();assert_ne!(another.id,task.id);
}
#[test]
fn source_owner_conflicts_cannot_silently_drop_requested_targets(){
 let root=tempfile::tempdir().unwrap();let mut q=Queue::open(root.path()).unwrap();q.enqueue(request("book-a")).unwrap();
 let mut changed=request("book-a");changed.targets.push("unit-c".into());assert!(q.enqueue(changed).is_err());
 assert!(q.enqueue(request("book-b")).is_ok());
}
#[test]
fn pause_races_and_two_connections_never_claim_or_finalize_work_twice(){
 let root=tempfile::tempdir().unwrap();let mut a=Queue::open(root.path()).unwrap();let mut b=Queue::open(root.path()).unwrap();
 let task=a.enqueue(request("book-a")).unwrap();a.claim().unwrap();assert!(b.claim().unwrap().is_none());
 b.pause(&task.id).unwrap();assert_eq!(a.read(&task.id).unwrap().state,"pausing");
 a.finish(&task.id,"ready",None).unwrap();assert_eq!(b.read(&task.id).unwrap().state,"paused");
 assert!(b.resume(&task.id).is_ok());let running=a.claim().unwrap().unwrap();assert_eq!(running.id,task.id);
 a.finish(&task.id,"failed",Some("network result uncertain".into())).unwrap();
 assert_eq!(b.read(&task.id).unwrap().error.as_deref(),Some("network result uncertain"));
}
#[test]
fn invalid_requests_cannot_escape_paths_and_snapshots_exclude_full_source(){
 let root=tempfile::tempdir().unwrap();let mut q=Queue::open(root.path()).unwrap();
 for owner in ["../escape","","a/b"]{assert!(q.enqueue(request(owner)).is_err());}
 let mut invalid=request("book-a");invalid.kind="tts-direct".into();assert!(q.enqueue(invalid).is_err());
 let task=q.enqueue(request("book-a")).unwrap();let v=serde_json::to_value(task).unwrap();
 assert!(v.get("paper").is_none()&&v.get("apiKey").is_none());assert_eq!(v["targets"],json!(["unit-a","unit-b"]));
 assert_eq!(q.list(10).unwrap().len(),1);
}
#[test]
fn queued_work_after_process_shutdown_is_paused_and_can_be_cancelled_without_spending(){
 let root=tempfile::tempdir().unwrap();let mut q=Queue::open(root.path()).unwrap();let task=q.enqueue(request("book-a")).unwrap();
 assert_eq!(q.recover_interrupted().unwrap(),1);assert_eq!(q.read(&task.id).unwrap().state,"paused");
 q.cancel(&task.id).unwrap();assert_eq!(q.read(&task.id).unwrap().state,"cancelled");
 let fresh=q.enqueue(request("book-a")).unwrap();assert_ne!(fresh.id,task.id);
 let mut bad=TaskRequest{kind:"cloud".into(),owner:"cloud-job".into(),targets:vec!["other".into()],title:"Invalid".into()};assert!(q.enqueue(bad.clone()).is_err());bad.targets=vec![bad.owner.clone()];assert!(q.enqueue(bad).is_ok());
}
