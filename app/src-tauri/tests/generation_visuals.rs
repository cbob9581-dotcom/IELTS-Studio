use ielts_core::cloud::{visuals,generation};
use serde_json::json;
#[test]
fn all_supported_visual_templates_render_and_can_export_review_gallery(){
 for kind in ["table","bar","line","pie","process","map","combined"]{let visual=visuals::template(kind).unwrap();let bytes=visuals::render(&visual).unwrap();assert!(bytes.len()>1000);if let Ok(dir)=std::env::var("IELTS_VISUAL_EXPORT_DIR"){std::fs::create_dir_all(&dir).unwrap();std::fs::write(std::path::Path::new(&dir).join(format!("{kind}.png")),bytes).unwrap();}}
}
#[test]
fn charts_use_one_numeric_dataset_and_reject_invalid_or_misleading_values(){
 for kind in ["bar","line","pie"]{let data=json!({"kind":kind,"title":"Library visits in 2024","unit":"thousands","categories":["North","South","East"],"series":[{"name":"2024","values":[10,20,30]}]});let png=visuals::render(&data).unwrap();assert!(png.starts_with(b"\x89PNG"));}
 let bad=json!({"kind":"bar","title":"Bad","unit":"people","categories":["A","B"],"series":[{"name":"year","values":[1]}]});assert!(visuals::render(&bad).is_err());
 let bad=json!({"kind":"pie","title":"Bad","unit":"people","categories":["A","B"],"series":[{"name":"year","values":[-1,0]}]});assert!(visuals::render(&bad).is_err());
}
#[test]
fn process_and_comparative_map_visuals_are_renderable_without_image_generation(){
 let process=json!({"kind":"process","title":"How paper is recycled","steps":[{"id":"a","label":"Collect paper"},{"id":"b","label":"Sort material"},{"id":"c","label":"Make pulp"}],"connections":[{"from":"a","to":"b"},{"from":"b","to":"c"}]});assert!(visuals::render(&process).is_ok());
 let map=json!({"kind":"map","title":"A park in 1990 and 2020","panels":[{"title":"1990","features":[{"label":"Trees","type":"park","x":0.1,"y":0.1,"width":0.3,"height":0.3}]},{"title":"2020","features":[{"label":"Library","type":"building","x":0.1,"y":0.1,"width":0.3,"height":0.3}]}]});assert!(visuals::render(&map).is_ok());
 let mut bad=process;bad["connections"][0]["to"]=json!("missing");assert!(visuals::render(&bad).is_err());
}
#[test]
fn blueprint_variants_preserve_all_score_positions_and_visual_prompts_match_selected_shape(){
 for seed in 0..3{for section in ["listening","reading"]{let max=if section=="listening"{4}else{3};let total:u64=(1..=max).map(|part|generation::question_plan_variant(section,part,None,seed).iter().map(|q|q["points"].as_u64().unwrap()).sum::<u64>()).sum();assert_eq!(total,40);}}
 let a=generation::question_plan_variant("reading",1,None,0);let b=generation::question_plan_variant("reading",1,None,1);assert_ne!(a,b);
 let prompt=generation::prompt_for_request("writing",1,None,"",&json!({"visualKind":"bar"})).unwrap();assert!(prompt.contains("categories")&&prompt.contains("series"));
}
