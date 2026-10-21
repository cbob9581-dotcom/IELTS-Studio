use ielts_core::{task_queue::{Queue,TaskRequest},task_policy::TaskPolicy};
#[test]
fn enqueueing_the_same_failed_scope_preserves_its_request_budget_and_checkpoints(){
 let root=tempfile::tempdir().unwrap();let mut q=Queue::open(root.path()).unwrap();let request=TaskRequest{kind:"cloud".into(),owner:"job".into(),targets:vec!["job".into()],title:"Generation".into()};let t=q.enqueue(request.clone()).unwrap();q.claim().unwrap();q.reserve_request(&t.id,"request-one","model",1).unwrap();q.finish(&t.id,"failed",Some("budget".into())).unwrap();let retry=q.enqueue(request).unwrap();assert_eq!(retry.id,t.id);assert_eq!(retry.usage.requests,1);q.claim().unwrap();assert!(q.reserve_request(&retry.id,"request-two","model",1).is_err());
}
#[test]
fn request_quota_is_durable_and_uncertain_calls_still_use_a_slot(){
 let root=tempfile::tempdir().unwrap();let mut q=Queue::open(root.path()).unwrap();let t=q.enqueue(TaskRequest{kind:"cloud".into(),owner:"job".into(),targets:vec!["job".into()],title:"Generation".into()}).unwrap();q.claim().unwrap();
 q.reserve_request(&t.id,"attempt-one","model",1).unwrap();assert!(q.reserve_request(&t.id,"attempt-two","model",1).is_err());
 q.record_usage("attempt-one",Some(12),Some(34)).unwrap();let stats=q.usage(&t.id).unwrap();assert_eq!(stats.requests,1);assert_eq!(stats.input_tokens,Some(12));assert_eq!(stats.output_tokens,Some(34));
 assert!(q.reserve_request(&t.id,"attempt-two","model",2).is_ok());assert_eq!(q.usage(&t.id).unwrap().requests,2);
}
#[test]
fn task_policy_has_safe_defaults_and_rejects_invalid_parallelism(){
 let root=tempfile::tempdir().unwrap();let mut policy=TaskPolicy::load(root.path()).unwrap();assert_eq!(policy.background_concurrency,2);assert_eq!(policy.max_text_requests,40);
 policy.background_concurrency=0;assert!(policy.save(root.path()).is_err());policy.background_concurrency=3;policy.max_text_requests=10;policy.save(root.path()).unwrap();assert_eq!(TaskPolicy::load(root.path()).unwrap().max_text_requests,10);
}
