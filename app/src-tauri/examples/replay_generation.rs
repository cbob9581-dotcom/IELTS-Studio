//! Offline diagnostics only: reads a saved response, never invokes a provider.
use ielts_core::cloud::generation;
use serde_json::{Value,json};
fn main()->Result<(),Box<dyn std::error::Error>>{
 let args:Vec<_>=std::env::args().collect();let mut v:Value=serde_json::from_slice(&std::fs::read(args.get(1).ok_or("response path required")?)?)?;let section=args.get(2).ok_or("section required")?;let part:u64=args.get(3).ok_or("part required")?.parse()?;
 let result=generation::normalize_generated(&mut v,section,part).and_then(|_|generation::validate_part(&v,section,part,None));
 println!("{}",json!({"passed":result.is_ok(),"error":result.err().map(|e|e.message),"questions":v["questions"].as_array().map(Vec::len),"marks":v["questions"].as_array().into_iter().flatten().map(|q|q["points"].as_u64().unwrap_or(0)).sum::<u64>(),"cloudRequests":0}));Ok(())
}
