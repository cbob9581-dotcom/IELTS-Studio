use ielts_core::source_library::audio_assets;
use serde_json::json;
#[test]
fn whole_test_plays_once_and_part_tracks_sort_by_explicit_order(){
 let docs=json!([{"id":"whole","state":"media","durationMs":10000,"binding":{"test":"Test 1","scope":"test","active":true,"order":0}}]);
 let list=audio_assets::playlist(docs.as_array().unwrap(),"Test 1",&[1,2,3,4]).unwrap();assert_eq!(list.len(),1);assert!(list[0]["part"].is_null());
 let docs=json!([{"id":"b","state":"media","durationMs":1000,"binding":{"test":"Test 1","part":1,"order":2}},{"id":"a","state":"media","durationMs":1000,"binding":{"test":"Test 1","part":1,"order":1}}]);
 let list=audio_assets::playlist(docs.as_array().unwrap(),"Test 1",&[1]).unwrap();assert_eq!(list[0]["assetId"],"a");assert_eq!(list[1]["assetId"],"b");assert!(audio_assets::playlist(docs.as_array().unwrap(),"Test 1",&[2]).is_err());
}
#[test]
fn conflicting_sources_and_invalid_ranges_are_rejected(){
 let docs=json!([{"id":"a","state":"media","binding":{"test":"Test 1","scope":"test"}},{"id":"b","state":"media","binding":{"test":"Test 1","part":1}}]);assert!(audio_assets::playlist(docs.as_array().unwrap(),"Test 1",&[1]).is_err());
 assert!(audio_assets::validate_binding(&json!({"test":"Test 1","scope":"test","segments":[{"part":1,"startMs":200,"endMs":100}]}),1000).is_err());
}
#[test]
fn publishing_one_part_does_not_include_other_part_tracks(){
 let docs=json!([{"id":"p1","state":"media","binding":{"test":"Test 1","part":1}},{"id":"p2","state":"media","binding":{"test":"Test 1","part":2}}]);let list=audio_assets::playlist(docs.as_array().unwrap(),"Test 1",&[2]).unwrap();assert_eq!(list.len(),1);assert_eq!(list[0]["assetId"],"p2");
}
#[test]
fn wav_decode_and_render_silence_have_measured_duration(){
 let wav=audio_assets::wav(&vec![0.1;2400],24000);let pcm=audio_assets::decode(&wav,"wav",10).unwrap();assert_eq!(pcm.len(),2400);
 assert!(audio_assets::decode(b"bad data","mp3",10).is_err());
 let events=json!([{"type":"speech","turnId":"t1"},{"type":"silence","durationMs":300},{"type":"speech","turnId":"t2"}]);
 let samples=audio_assets::render(events.as_array().unwrap(),&std::collections::HashMap::from([("t1".to_string(),pcm.clone()),("t2".into(),pcm)])).unwrap();assert_eq!(samples.len(),12000);
}
#[test]
fn partial_listening_publication_uses_measured_track_duration(){
 let p=json!({"id":"project","title":"Book","units":[{"id":"u","test":"Test 1","section":"listening","part":1,"role":"questions","state":"complete","confirmed":true,"result":{"section":{"questions":[{"id":"q","number":1,"part":1,"kind":"gap_fill","points":1,"prompt":"A ___","evidence":[],"answer":{"accepted":["tree"],"caseSensitive":false},"answerSource":{"quote":"1 tree"}}],"paragraphs":[]}}}],"documents":[{"id":"a","state":"media","audioVerified":true,"durationMs":2500,"extension":"wav","path":"media/a.wav","sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","binding":{"test":"Test 1","part":1}}]});
 let paper=ielts_core::source_library::publication::assemble(&p,"Test 1",&["u".into()],1).unwrap();assert_eq!(paper["sections"][0]["durationMs"],2500);
}
