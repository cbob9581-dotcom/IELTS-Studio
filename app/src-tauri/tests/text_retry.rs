use ielts_core::cloud::jobs;
use serde_json::json;
#[test]
fn explicit_assessment_retry_preserves_completed_tasks_and_raw_recordings(){
 let t=tempfile::tempdir().unwrap();let mut j=jobs::create(t.path(),"assessment",json!({"attemptId":"attempt"})).unwrap();let dir=jobs::dir(t.path(),j["id"].as_str().unwrap()).unwrap();std::fs::write(dir.join("attempt.json"),r#"{"section":"writing"}"#).unwrap();j["state"]=json!("failed");j["stage"]=json!(1);j["evaluations"]=json!([{"task":1}]);std::fs::write(dir.join("writing-0-response.txt"),"completed").unwrap();std::fs::write(dir.join("writing-1-response.request.json"),"unknown").unwrap();
 jobs::retry_assessment(t.path(),&mut j).unwrap();assert_eq!(j["evaluations"].as_array().unwrap().len(),1);assert_eq!(j["stage"],1);assert!(dir.join("writing-0-response.txt").exists());assert!(!dir.join("writing-1-response.request.json").exists());assert_eq!(j["state"],"pending");assert!(dir.join("attempt.json").exists());
}
#[test]
fn retry_archives_only_the_failed_text_part_and_preserves_completed_steps_and_media(){
 let tmp=tempfile::tempdir().unwrap();let mut j=jobs::create(tmp.path(),"generation",json!({"section":"reading"})).unwrap();let dir=jobs::dir(tmp.path(),j["id"].as_str().unwrap()).unwrap();
 j["state"]=json!("failed");j["stage"]=json!(1);std::fs::write(dir.join("part-0.json"),"completed").unwrap();std::fs::write(dir.join("part-1-raw.json"),"rejected").unwrap();std::fs::write(dir.join("part-1-raw.repair.json"),"rejected correction").unwrap();std::fs::create_dir(dir.join("media")).unwrap();std::fs::write(dir.join("media/user.wav"),"original").unwrap();
 jobs::retry_text(tmp.path(),&mut j).unwrap();assert_eq!(j["stage"],1);assert_eq!(j["state"],"pending");assert!(dir.join("part-0.json").is_file());assert!(dir.join("media/user.wav").is_file());assert!(!dir.join("part-1-raw.json").exists());assert!(dir.join(j["lastTextRetryArchive"].as_str().unwrap()).join("part-1-raw.json").is_file());
}
