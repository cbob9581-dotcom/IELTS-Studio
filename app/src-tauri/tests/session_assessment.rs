use ielts_core::cloud::assessment;
use serde_json::json;
#[test]
fn whole_session_scores_are_equal_criterion_means_and_require_recording_evidence(){
 let recordings=json!([{"turnId":"t1","recordingId":"r1","durationSeconds":60.0,"signalSeconds":40.0}]);
 let mut result=json!({"audioUsable":true,"summary":"Practice only","improvements":[],"criteria":[{"name":"fluency","band":6.0,"evidence":["t1 00:10 pause"],"feedback":"pause"},{"name":"lexical","band":7.0,"evidence":["t1 vocabulary"],"feedback":"words"},{"name":"grammar","band":7.0,"evidence":["t1 grammar"],"feedback":"grammar"},{"name":"pronunciation","band":6.0,"evidence":["t1 00:12 stress"],"feedback":"stress"}],"evidenceReferences":[{"turnId":"t1","recordingId":"r1","startSeconds":10.0,"endSeconds":13.0}]});
 assert_eq!(assessment::validate_session(&result,&recordings).unwrap(),6.5);
 result["evidenceReferences"][0]["recordingId"]=json!("made-up");assert!(assessment::validate_session(&result,&recordings).is_err());
 result["evidenceReferences"][0]["recordingId"]=json!("r1");result["evidenceReferences"][0]["endSeconds"]=json!(90.0);assert!(assessment::validate_session(&result,&recordings).is_err());
 assert!(assessment::session_audio_supported("qwen3.8-omni-flash",12));assert!(!assessment::session_audio_supported("qwen3-omni-flash",12));
}
