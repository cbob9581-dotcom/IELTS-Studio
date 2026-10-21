use ielts_core::{cloud::jobs,papers};
use serde_json::{json,Value};
fn draft()->Value{let p=json!({"title":"Reviewed text","assets":[],"sections":[]});let h=papers::hash(&serde_json::to_vec(&p).unwrap());json!({"kind":"generation","state":"review_required","paper":p,"validation":{"pass":true,"issues":[]},"validationHash":h,"textFingerprint":"text-v1","audioPending":true})}
#[test]
fn old_jobs_and_client_flags_never_authorize_paid_tts(){
 let mut j=draft();j["request"]=json!({"audioEnabled":true});
 assert!(jobs::check_audio_authorization(&j,"text-v1").is_err());
}
#[test]
fn explicit_authorization_requires_approved_text_and_is_invalidated_by_changes(){
 let mut j=draft();jobs::authorize_audio(&mut j).unwrap();
 jobs::check_audio_authorization(&j,"text-v1").unwrap();
 assert!(jobs::check_audio_authorization(&j,"text-v2").is_err());
 j["paper"]["title"]=json!("Changed after approval");
 assert!(jobs::check_audio_authorization(&j,"text-v1").is_err());
}
#[test]
fn rejected_audit_cannot_authorize_audio(){
 let mut j=draft();j["validation"]["pass"]=json!(false);
 assert!(jobs::authorize_audio(&mut j).is_err());
 assert!(j.get("audioAuthorization").is_none());
}
