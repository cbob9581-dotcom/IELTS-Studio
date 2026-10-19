use ielts_core::cloud::{assessment, generation, ingestion, jobs};
use serde_json::json;

#[test]
fn damaged_task_does_not_hide_other_papers() {
    let tmp = tempfile::tempdir().unwrap();
    let job = jobs::create(tmp.path(), "generation", json!({"section":"reading"})).unwrap();
    let bad = jobs::dir(tmp.path(), "broken").unwrap();
    std::fs::create_dir_all(&bad).unwrap();
    std::fs::write(bad.join("job.json"), b"{").unwrap();
    assert_eq!(jobs::list(tmp.path()).unwrap().len(), 1);
    assert_eq!(jobs::list(tmp.path()).unwrap()[0]["id"], job["id"]);
}
#[test]
fn new_task_categories_and_legacy_tasks_are_both_readable_and_summaries_are_small() {
    let tmp = tempfile::tempdir().unwrap();
    let j = jobs::create(tmp.path(), "import", json!({"section":"reading"})).unwrap();
    let path = jobs::dir(tmp.path(), j["id"].as_str().unwrap()).unwrap();
    assert_eq!(path.parent().unwrap().file_name().unwrap(), "import");
    let legacy = tmp.path().join("cloud-jobs/old-job");
    std::fs::create_dir_all(&legacy).unwrap();
    std::fs::write(
        legacy.join("job.json"),
        br#"{"id":"old-job","kind":"generation","stage":1}"#,
    )
    .unwrap();
    assert_eq!(jobs::list(tmp.path()).unwrap().len(), 2);
    assert_eq!(jobs::load(tmp.path(), "old-job").unwrap()["stage"], 1);
    let mut heavy = j;
    heavy["sourcePreview"] = json!("x".repeat(100000));
    heavy["paper"] = json!({"title":"A draft","sections":["huge"]});
    assert!(jobs::summary(&heavy).get("sourcePreview").is_none());
    assert!(jobs::summary(&heavy).get("paper").is_none());
    assert_eq!(jobs::summary(&heavy)["paperTitle"], "A draft");
}

#[test]
fn separately_extracted_sections_cannot_collide_on_common_question_ids() {
    let mut reading = json!({"paper":{"sections":[{"section":"reading","paragraphs":[{"id":"p1"}],"questions":[{"id":"q1","number":1}]}]},"mapping":[{"questionId":"q1","sourceQuote":"Answer 1: solar"}]});
    let mut writing = json!({"paper":{"sections":[{"section":"writing","paragraphs":[],"questions":[{"id":"q1","number":1}]}]},"mapping":[]});
    ingestion::namespace_section(&mut reading, "reading").unwrap();
    ingestion::namespace_section(&mut writing, "writing").unwrap();
    assert_ne!(
        reading["paper"]["sections"][0]["questions"][0]["id"],
        writing["paper"]["sections"][0]["questions"][0]["id"]
    );
    assert_eq!(
        reading["mapping"][0]["questionId"],
        reading["paper"]["sections"][0]["questions"][0]["id"]
    );
    let saved = reading.clone();
    ingestion::namespace_section(&mut reading, "reading").unwrap();
    assert_eq!(saved, reading);
}

#[test]
fn generation_respects_part_and_kind_and_originality() {
    let p = generation::prompt("reading", 2, Some("gap_fill"), "cities").unwrap();
    assert!(p.contains("gap_fill") && p.contains("original") && p.contains("2"));
    assert!(generation::prompt("reading", 4, None, "").is_err());
    assert!(generation::prompt("listening", 1, Some("true_false_not_given"), "").is_err());
}
#[test]
fn rejects_wrong_question_count_and_invented_evidence() {
    let mut v = json!({"paragraphs":[{"id":"p1","text":"A factual passage.","part":1}],"questions":[{"id":"q1","part":1,"kind":"gap_fill","prompt":"Missing word", "number":1,"points":1,"evidence":["not present"],"answer":{"accepted":["word"],"caseSensitive":false,"maxWords":1}}]});
    assert!(generation::validate_part(&v, "reading", 1, None).is_err());
    v["questions"][0]["evidence"] = json!(["A factual passage."]);
    assert!(generation::validate_part(&v, "reading", 1, None).is_err());
}
#[test]
fn rejects_text_only_pronunciation_and_weights_writing() {
    let v = json!({"criteria":[{"name":"pronunciation","band":7.0,"evidence":["good"]}]});
    assert!(assessment::validate(&v, "speaking", false).is_err());
    assert_eq!(assessment::writing_band(6.0, 7.5), 7.0);
}
#[test]
fn rejects_missing_rubric_and_out_of_range_scores() {
    assert!(assessment::validate(&json!({"criteria":[]}), "writing", false).is_err());
    let criteria: Vec<_> = ["task", "coherence", "lexical", "grammar"]
        .iter()
        .map(|n| json!({"name":n,"band":12,"evidence":["quote"],"feedback":"explain"}))
        .collect();
    assert!(assessment::validate(&json!({"criteria":criteria}), "writing", false).is_err());
}
#[test]
fn zip_paths_cannot_escape_or_use_windows_special_paths() {
    for x in [
        "../x",
        "C:/x",
        "a/../../x",
        "a\\x",
        "CON.txt",
        "a/NUL",
        "a/x:ads",
    ] {
        assert!(!ingestion::safe_entry(x), "{x}");
    }
    assert!(ingestion::safe_entry("images/figure-1.png"));
}
#[test]
fn sse_collects_only_answer_and_requires_finish() {
    let s="data: {\"choices\":[{\"delta\":{\"reasoning_content\":\"private\",\"content\":\"{\\\"ok\\\":true}\"},\"finish_reason\":null}]}\n\ndata: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\ndata: [DONE]\n";
    assert_eq!(ielts_core::cloud::parse_sse(s).unwrap(), json!({"ok":true}));
    assert!(ielts_core::cloud::parse_sse("data: [DONE]\n").is_err());
}
#[test]
fn durable_job_reloads_completed_checkpoint_without_secret() {
    let tmp = tempfile::tempdir().unwrap();
    let job = jobs::create(tmp.path(), "generation", json!({"section":"reading"})).unwrap();
    let mut saved = job.clone();
    saved["stage"] = json!(1);
    jobs::save(tmp.path(), &saved).unwrap();
    assert_eq!(
        jobs::load(tmp.path(), job["id"].as_str().unwrap()).unwrap()["stage"],
        1
    );
    assert!(jobs::load(tmp.path(), "../secret").is_err());
}
#[test]
fn diagram_is_generated_without_printing_the_hidden_answers() {
    let v = json!({"title":"A simple water cycle","items":[{"id":"a","text":"Collection","x":0.2,"y":0.3},{"id":"b","text":"Question 1","x":0.7,"y":0.3}],"connections":[{"from":"a","to":"b"}]});
    let bytes = generation::diagram_png(&v).unwrap();
    assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
    assert!(
        generation::diagram_png(&json!({"items":[{"id":"a","text":"bad","x":2,"y":0}]})).is_err()
    );
}
#[test]
fn import_answers_require_an_exact_source_key_quote() {
    let p = json!({"sections":[{"questions":[{"id":"q1","kind":"gap_fill","answer":{"accepted":["library"]},"evidence":["library appears in the passage"]}]}]});
    assert!(
        ingestion::verify_answer_sources(&p, "A library is open. Answer key: 1 library").is_err()
    );
    let mut p = p;
    p["sections"][0]["questions"][0]["evidence"] = json!(["ANSWER_SOURCE: Answer key: 1 library"]);
    assert!(
        ingestion::verify_answer_sources(&p, "A library is open. Answer key: 1 library").is_ok()
    );
}
#[test]
fn source_key_quotes_accept_presentation_quotes_but_never_nonexistent_content() {
    let mut p = json!({"sections":[{"questions":[{"id":"reading-q1","kind":"gap_fill","answer":{"accepted":["solar"]},"evidence":["ANSWER_SOURCE: \"Answer key: Reading 1: solar.\""]}]}]});
    assert!(ingestion::verify_answer_sources(&p, "Answer key: Reading 1: solar.").is_ok());
    p["sections"][0]["questions"][0]["evidence"] =
        json!(["ANSWER_SOURCE: \"Answer key: Reading 1: wind.\""]);
    assert!(ingestion::verify_answer_sources(&p, "Answer key: Reading 1: solar.").is_err());
}
#[test]
fn academic_table_wraps_long_headers_and_preserves_typographic_ranges() {
    let v = json!({"title":"Public transport use","columns":["Transport mode","Average satisfaction score (1–5)"],"rows":[["Metro","4.5"],["Bus","3.6"],["Ferry","4.0"]]});
    assert!(generation::table_png(&v).unwrap().starts_with(b"\x89PNG"));
    let mut task = json!({"questions":[{"prompt":"Write at least 250 words.","answer":{"accepted":[],"caseSensitive":false}}]});
    generation::normalize_metadata(&mut task, "writing", 2).unwrap();
    assert_eq!(task["questions"][0]["minWords"], 250);
    assert!(task["questions"][0]["answer"].is_null());
}
#[test]
fn high_rate_long_recording_keeps_original_and_fits_audio_model_limit() {
    let mut b = vec![0u8; 44 + 48000 * 2 * 120];
    let len = b.len() as u32;
    b[..4].copy_from_slice(b"RIFF");
    b[4..8].copy_from_slice(&(len - 8).to_le_bytes());
    b[8..12].copy_from_slice(b"WAVE");
    b[12..16].copy_from_slice(b"fmt ");
    b[16..20].copy_from_slice(&16u32.to_le_bytes());
    b[20..22].copy_from_slice(&1u16.to_le_bytes());
    b[22..24].copy_from_slice(&1u16.to_le_bytes());
    b[24..28].copy_from_slice(&48000u32.to_le_bytes());
    b[28..32].copy_from_slice(&96000u32.to_le_bytes());
    b[32..34].copy_from_slice(&2u16.to_le_bytes());
    b[34..36].copy_from_slice(&16u16.to_le_bytes());
    b[36..40].copy_from_slice(b"data");
    b[40..44].copy_from_slice(&(len - 44).to_le_bytes());
    let original = ielts_core::papers::hash(&b);
    let copy = assessment::audio_for_model(&b).unwrap();
    assert!(copy.len() < 7 * 1024 * 1024);
    assert_eq!(original, ielts_core::papers::hash(&b));
    assert_eq!(u32::from_le_bytes(copy[24..28].try_into().unwrap()), 24000);
    ielts_core::speech::validate_wav(&copy).unwrap();
    b[40..44].copy_from_slice(&10u32.to_le_bytes());
    assert!(ielts_core::speech::validate_wav(&b).is_err());
}
#[test]
fn word_limits_in_model_top_level_are_moved_to_answer_without_changing_content() {
    let mut v = json!({"questions":[{"kind":"gap_fill","maxWords":2,"maxNumbers":1,"answer":{"accepted":["pocket parks"],"caseSensitive":false}}]});
    generation::normalize_metadata(&mut v, "reading", 2).unwrap();
    assert_eq!(v["questions"][0]["answer"]["maxWords"], 2);
    assert!(v["questions"][0].get("maxWords").is_none());
}
#[test]
fn silent_recording_is_rejected_before_paying_for_audio_assessment() {
    let mut bytes = include_bytes!("../../test/fixtures/media/three-test-tones.wav").to_vec();
    bytes[44..].fill(0);
    assert!(assessment::signal_seconds(&bytes).is_err());
}
#[test]
fn listening_narrator_is_separate_from_the_two_conversation_roles() {
    let text = "word ".repeat(80);
    let transcript: Vec<_> = [2, 0, 1, 0, 1, 0, 2]
        .iter()
        .map(|n| json!({"speaker":n,"text":text}))
        .collect();
    let qs:Vec<_>=(1..=10).map(|n|json!({"id":format!("q{n}"),"part":1,"number":n,"kind":"gap_fill","prompt":"Write ONE WORD","points":1,"evidence":["word word"],"answer":{"accepted":["word"],"caseSensitive":false,"maxWords":1}})).collect();
    assert!(generation::validate_part(
        &json!({"paragraphs":[],"transcript":transcript,"questions":qs}),
        "listening",
        1,
        Some("gap_fill")
    )
    .is_ok());
}

#[test]
fn unchanged_review_cannot_override_failed_content_validation() {
    let mut j = json!({"paper":{"title":"same"},"validation":{"pass":false},"humanEditedAt":1});
    jobs::review_edit(&mut j, json!({"title":"same"}));
    assert!(j["humanEditedAt"].is_null());
    jobs::review_edit(&mut j, json!({"title":"changed"}));
    assert!(j["humanEditedAt"].is_number());
}
#[test]
fn speaking_summary_uses_signal_and_skips_insufficient_rounds() {
    let criteria = |n| json!({"criteria":[{"band":n},{"band":n},{"band":n},{"band":n}]});
    let a = json!({"durationSeconds":60,"signalSeconds":2,"result":criteria(9)});
    assert!(assessment::speaking_band(&[a.clone()]).is_err());
    let b = json!({"durationSeconds":35,"signalSeconds":30,"result":criteria(5)});
    let short =
        json!({"durationSeconds":2,"signalSeconds":1,"skipped":true,"result":{"criteria":[]}});
    assert_eq!(assessment::speaking_band(&[a, b, short]).unwrap(), 5.5);
}

#[test]
fn verdict_is_bound_to_exact_paper_and_media() {
    let paper = json!({"title":"original","assets":[]});
    let hash = ielts_core::papers::hash(&serde_json::to_vec(&paper).unwrap());
    let mut j = json!({"paper":paper,"validation":{"pass":true,"issues":[]},"validationHash":hash});
    assert!(jobs::content_approved(&j));
    j["paper"]["title"] = json!("changed");
    assert!(!jobs::content_approved(&j));
}

#[test]
fn lightweight_status_keeps_titles_and_recovers_from_corrupt_cache() {
    let tmp = tempfile::tempdir().unwrap();
    let mut j = jobs::create(tmp.path(), "generation", json!({"section":"reading"})).unwrap();
    j["paper"] = json!({"title":"Human readable title"});
    jobs::save(tmp.path(), &j).unwrap();
    let summary = jobs::summary(&jobs::list(tmp.path()).unwrap()[0]);
    assert_eq!(summary["paperTitle"], "Human readable title");
    std::fs::write(
        jobs::dir(tmp.path(), j["id"].as_str().unwrap())
            .unwrap()
            .join("status.json"),
        b"{",
    )
    .unwrap();
    assert_eq!(jobs::list(tmp.path()).unwrap().len(), 1);
}
