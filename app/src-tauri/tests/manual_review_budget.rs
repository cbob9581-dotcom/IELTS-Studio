use ielts_core::{cloud::{jobs,generation_workflow::{self,Providers},request::{BudgetContext,BUDGET}},configuration::ProviderConfig,contracts::Result,task_queue::{Queue,TaskRequest}};
use serde_json::json;
struct Services;
impl Providers for Services{fn provider(&self,_:&str)->Result<(ProviderConfig,String)>{Ok((ProviderConfig{base_url:"https://example.invalid/v1".into(),model:"fixture".into(),..Default::default()},"unused".into()))}}
#[tokio::test]
async fn scheduled_manual_review_cannot_bypass_an_existing_exhausted_budget(){
 let t=tempfile::tempdir().unwrap();let mut j=jobs::create(t.path(),"generation",json!({"section":"reading","part":1})).unwrap();j["state"]=json!("review_required");j["paper"]=serde_json::from_str(include_str!("../../test/fixtures/dev-reading.json")).unwrap();jobs::schedule_recheck(&mut j).unwrap();assert_eq!(j["state"],"pending");
 let dir=jobs::dir(t.path(),j["id"].as_str().unwrap()).unwrap();std::fs::write(dir.join("part-0.json"),r#"{"paragraphs":[],"questions":[],"transcript":[]}"#).unwrap();let mut q=Queue::open(t.path()).unwrap();let id=j["id"].as_str().unwrap().to_string();let task=q.enqueue(TaskRequest{kind:"cloud".into(),owner:id.clone(),targets:vec![id],title:"Manual review".into()}).unwrap();q.claim().unwrap();q.reserve_request(&task.id,"previous-request","fixture",1).unwrap();let context=BudgetContext{root:t.path().to_path_buf(),task_id:task.id.clone(),limit:1};let error=BUDGET.scope(context,generation_workflow::step(&Services,t.path(),&mut j)).await.unwrap_err();assert_eq!(error.code,"task_budget");assert_eq!(q.usage(&task.id).unwrap().requests,1);
}
