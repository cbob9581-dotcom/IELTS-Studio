use ielts_core::calibration::measure;
use serde_json::json;
#[test]
fn reports_held_out_accuracy_and_repeat_variation_without_counting_repeats_as_new_people(){
 let sample=|id:&str,reference:f64,predicted:f64|json!({"id":id,"referenceBand":reference,"predictedBand":predicted,"model":"model-a","protocol":"session-v1","referenceSource":"independent human annotation"});
 let result=measure(&json!([sample("one",6.,6.),sample("one",6.,7.),sample("two",7.,6.5)])).unwrap();let group=&result["groups"][0];assert_eq!(group["uniqueSamples"],2);assert_eq!(group["runs"],3);assert_eq!(group["meanAbsoluteError"],0.5);assert_eq!(group["withinHalfBand"],0.75);assert_eq!(group["repeatStandardDeviation"],0.5);assert_eq!(group["targetMet"],false);
}
#[test]
fn rejects_invalid_scores_and_disagreeing_reference_labels(){
 let sample=json!({"id":"one","referenceBand":6.,"predictedBand":10.,"model":"a","protocol":"v1","referenceSource":"human"});assert!(measure(&json!([sample])).is_err());assert!(measure(&json!([])).is_err());
}
