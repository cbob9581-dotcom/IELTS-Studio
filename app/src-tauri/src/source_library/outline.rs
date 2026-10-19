//! Printed book headings take precedence over window-based AI classification.
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

fn number_after(s: &str, prefix: &str) -> Option<u64> {
    let rest = s.split_once(prefix)?.1.trim_start();
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}
fn short_test(s: &str) -> Option<u64> {
    if s.len() > 85 || !s.starts_with("TEST") { return None; }
    number_after(s, "TEST").filter(|n| (1..=99).contains(n))
}
pub fn derive(blocks: &[Value], roles: &Value) -> Vec<Value> {
    let analysis = roles.as_array().is_some_and(|r| !r.iter().any(|x| x=="mixed" || x=="questions") && r.iter().any(|x| x=="explanations" || x=="essays"));
    let mut skip = BTreeSet::new();
    let mut page_tests = BTreeMap::new();
    for b in blocks {
        let s=b["text"].as_str().unwrap_or("").trim().to_uppercase();
        let page=b["page"].as_u64().unwrap_or(0);
        if ["CONTENTS","目录","目录CONTENTS","前言PREFACE","PREFACE","INTRODUCTION"].contains(&s.as_str()) {skip.insert(page);}
        if let Some(n)=short_test(&s) {page_tests.insert(page,n);}
    }
    let mut test=0; let mut section=String::new(); let mut part=0;
    let mut region=if analysis {"explanations"}else{"questions"};
    let mut role=region; let mut last_page=None; let mut out:Vec<Value>=vec![];
    for (i,b) in blocks.iter().enumerate() {
        let page=b["page"].as_u64().unwrap_or(0);
        if skip.contains(&page) { continue; }
        let s=b["text"].as_str().unwrap_or("").trim().to_uppercase();
        if last_page!=Some(page) {
            if region!="transcript" {if let Some(&n)=page_tests.get(&page) {if n!=test {test=n;section.clear();part=0;role=region;}}}
            last_page=Some(page);
        }
        if (s=="AUDIOSCRIPTS" || s=="AUDIO SCRIPTS") && region!="transcript" {region="transcript";role=region;section="listening".into();part=0;}
        if s.contains("LISTENING AND READING ANSWER") {region="answers";role=region;section.clear();part=0;}
        if s.starts_with("SAMPLE WRITING ANSWER") {region="essays";role=region;section="writing".into();part=0;}
        if let Some(n)=short_test(&s) {if n!=test {test=n;section=if region=="transcript" {"listening".into()}else{String::new()};part=0;role=region;}}
        let heading=s.len()<100||(region=="answers"&&s.starts_with("READING PASSAGE"));
        let passage=if heading {number_after(&s,"READING PASSAGE")}else{None};
        let task=if heading {number_after(&s,"WRITING TASK")}else{None};
        let generic=if heading&&s.starts_with("PART") {number_after(&s,"PART")}else{None};
        let mut next=None;
        if passage.is_some() {next=Some("reading");}
        else if task.is_some() {next=Some("writing");}
        else if s.len()<65 {for (heading,key) in [("LISTENING","listening"),("READING","reading"),("WRITING","writing"),("SPEAKING","speaking")] {if s==heading || (s.starts_with("TEST")&&s.ends_with(heading)) {next=Some(key);break;}}}
        if let Some(next)=next {if section!=next {section=next.into();part=0;role=region;}}
        // Many answer keys list 1–40 under LISTENING/READING without Part headings.
        if region=="answers"&&!section.is_empty()&&part==0 {part=1;}
        if let Some(n)=passage.or(task).or(generic.filter(|_|section=="listening"||section=="speaking")) {
            if (1..=4).contains(&n) {part=n;role=region;}
        }
        if region=="essays" && s.contains("EXAMINER") && s.contains("COMMENT") {role="explanations";}
        if test==0 || section.is_empty() || part==0 {continue;}
        if section=="writing"&&part>2 || section=="reading"&&part>3 || section=="speaking"&&part>3 {continue;}
        let scope=json!({"test":format!("Test {test}"),"section":section,"part":part,"role":role,"start":i,"end":i,"title":format!("Test {test} · {section} · {part} · {role}")});
        if let Some(previous)=out.last_mut().filter(|p| ["test","section","part","role"].iter().all(|k|p[*k]==scope[*k])) {previous["end"]=json!(i);}
        else {out.push(scope);}
    }
    out
}
