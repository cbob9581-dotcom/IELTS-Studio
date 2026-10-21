use ielts_core::cloud::request;
use serde_json::json;
#[tokio::test]
async fn a_saved_response_is_reused_but_an_uncertain_request_is_never_resent(){
 let root=tempfile::tempdir().unwrap();let path=root.path().join("response.txt");let identity=json!({"model":"a","promptVersion":2});
 let bytes=request::recorded(&path,&identity,||async{Ok(b"response".to_vec())}).await.unwrap();assert_eq!(bytes,b"response");
 request::recorded(&path,&identity,||async{panic!("saved response must be reused")}).await.unwrap();
 assert!(request::recorded(&path,&json!({"model":"b"}),||async{panic!("parameter changes cannot silently replace a paid request")}).await.is_err());
 let uncertain=root.path().join("uncertain.txt");assert!(request::recorded(&uncertain,&identity,||async{Err(ielts_core::cloud::error("read timeout"))}).await.is_err());
 assert!(request::recorded(&uncertain,&identity,||async{panic!("unknown provider result must not automatically retry")}).await.is_err());
}
