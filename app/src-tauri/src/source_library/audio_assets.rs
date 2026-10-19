use super::err;
use crate::contracts::Result;
use serde_json::{json,Value};
use std::{collections::HashMap,io::Cursor};
use symphonia::core::{audio::SampleBuffer,codecs::DecoderOptions,formats::FormatOptions,io::MediaSourceStream,meta::MetadataOptions,probe::Hint};

pub fn decode(bytes:&[u8],ext:&str,max_seconds:u64)->Result<Vec<f32>>{decode_inner(bytes,ext,max_seconds,true).map(|x|x.0)}
pub fn duration(bytes:&[u8],ext:&str)->Result<u64>{decode_inner(bytes,ext,7200,false).map(|x|x.1)}
fn decode_inner(bytes:&[u8],ext:&str,max_seconds:u64,collect:bool)->Result<(Vec<f32>,u64)>{
 let mut hint=Hint::new();hint.with_extension(ext);
 let stream=MediaSourceStream::new(Box::new(Cursor::new(bytes.to_vec())),Default::default());
 let probed=symphonia::default::get_probe().format(&hint,stream,&FormatOptions{enable_gapless:true,..Default::default()},&MetadataOptions::default()).map_err(|_|err("音频格式无法识别，请使用可播放的 MP3、PCM WAV 或 Vorbis OGG"))?;
 let mut format=probed.format;
 let track=format.default_track().ok_or_else(||err("音频没有可用声轨"))?;let track_id=track.id;
 let mut decoder=symphonia::default::get_codecs().make(&track.codec_params,&DecoderOptions::default()).map_err(|_|err("音频编码暂不支持，请转换为 MP3 或 PCM WAV 后上传"))?;
 let mut out=vec![];let mut seconds=0.0;
 loop {
  let packet=match format.next_packet(){Ok(p)=>p,Err(symphonia::core::errors::Error::IoError(e)) if e.kind()==std::io::ErrorKind::UnexpectedEof=>break,Err(_)=>return Err(err("音频读取失败，文件可能损坏"))};
  if packet.track_id()!=track_id{continue;}
  let decoded=decoder.decode(&packet).map_err(|_|err("音频解码失败，文件可能损坏或编码不受支持"))?;
  let rate=decoded.spec().rate;let channels=decoded.spec().channels.count();if rate==0||channels==0{return Err(err("音频采样信息无效"));}
  let frames=decoded.frames();seconds+=frames as f64/rate as f64;if seconds>max_seconds as f64{return Err(err(format!("音频超过允许时长 {max_seconds} 秒，请拆分文件")));}
  if collect {
   let mut buf=SampleBuffer::<f32>::new(decoded.capacity() as u64,*decoded.spec());buf.copy_interleaved_ref(decoded);let samples=buf.samples();
   let n=((frames as f64*24000.0/rate as f64).round()) as usize;
   for j in 0..n {let x=j as f64*rate as f64/24000.0;let a=(x as usize).min(frames.saturating_sub(1));let b=(a+1).min(frames.saturating_sub(1));let frac=(x-a as f64) as f32;let mono=|i:usize|samples[i*channels..(i+1)*channels].iter().sum::<f32>()/channels as f32;out.push(mono(a)*(1.0-frac)+mono(b)*frac);}
  }
 }
 if seconds<=0.0{return Err(err("音频为空，没有可播放内容"));}Ok((out,(seconds*1000.0).round() as u64))
}
pub fn wav(samples:&[f32],rate:u32)->Vec<u8>{
 let len=(samples.len()*2) as u32;let mut b=vec![];b.extend(b"RIFF");b.extend((len+36).to_le_bytes());b.extend(b"WAVEfmt ");b.extend(16u32.to_le_bytes());b.extend(1u16.to_le_bytes());b.extend(1u16.to_le_bytes());b.extend(rate.to_le_bytes());b.extend((rate*2).to_le_bytes());b.extend(2u16.to_le_bytes());b.extend(16u16.to_le_bytes());b.extend(b"data");b.extend(len.to_le_bytes());for x in samples {b.extend(((x.clamp(-1.0,1.0)*32767.0) as i16).to_le_bytes());}b
}
pub fn render(events:&[Value],segments:&HashMap<String,Vec<f32>>)->Result<Vec<f32>>{
 let mut samples=vec![];
 for e in events {match e["type"].as_str(){Some("speech")=>samples.extend_from_slice(segments.get(e["turnId"].as_str().unwrap_or("")).ok_or_else(||err("正文片段未生成，已保留完成片段，请继续"))?),Some("silence")=>{let ms=e["durationMs"].as_u64().filter(|n|*n<=120000).ok_or_else(||err("停顿无效"))?;samples.resize(samples.len()+ms as usize*24,0.0);},_=>return Err(err("时间线事件无效"))}if samples.len()>(100*1024*1024-44)/2{return Err(err("单 Part 音轨超过100MB，请拆分正文或缩短停顿；已生成片段保留"));}}
 Ok(samples)
}
pub fn validate_binding(b:&Value,duration:u64)->Result<()> {
 if b["test"].as_str().is_none_or(|s|s.trim().is_empty()||s.len()>100){return Err(err("请填写 Test"));}
 if b["scope"]!="test"&&!(1..=4).contains(&b["part"].as_u64().unwrap_or(0)){return Err(err("请选择整套或 Part 1–4"));}
 if b["order"].as_u64().unwrap_or(0)>10000{return Err(err("播放顺序须在0–10000之间"));}
 let mut previous=0;let mut seen=std::collections::HashSet::new();
 for s in b["segments"].as_array().into_iter().flatten(){let p=s["part"].as_u64().unwrap_or(0);let start=s["startMs"].as_u64().unwrap_or(u64::MAX);let end=s["endMs"].as_u64().unwrap_or(0);if !(1..=4).contains(&p)||!seen.insert(p)||start<previous||end<=start||end>duration{return Err(err("Part 时间段需按时间排序、不重叠，并且在音频时长内"));}previous=end;}
 Ok(())
}
pub fn playlist(docs:&[Value],test:&str,required:&[u64])->Result<Vec<Value>>{
 let mut selected:Vec<_>=docs.iter().filter(|d|d["state"]=="media"&&d["binding"]["test"]==test&&d["binding"]["active"]!=false&&(d["binding"]["scope"]=="test"||required.contains(&d["binding"]["part"].as_u64().unwrap_or(0)))).collect();
 selected.sort_by_key(|d|d["binding"]["order"].as_u64().unwrap_or(0));
 if selected.is_empty(){return Err(err("听力尚未选择音频：可从真题脚本合成、用户文本合成，或上传音频后绑定"));}
 let whole=selected.iter().filter(|d|d["binding"]["scope"]=="test").count();
 if whole>0&&selected.len()>1{return Err(err("整套音频与其他音频重复覆盖，请只启用一个来源方案"));}
 let mut out=vec![];
 for d in &selected {let b=&d["binding"];validate_binding(b,d["durationMs"].as_u64().unwrap_or(0))?;out.push(json!({"assetId":d["id"],"part":if b["scope"]=="test"{Value::Null}else{b["part"].clone()},"segments":b["segments"].as_array().cloned().unwrap_or_default(),"origin":d["origin"].as_str().unwrap_or("uploaded"),"durationMs":d["durationMs"]}));}
 if whole==0{for part in required {let tracks:Vec<_>=selected.iter().filter(|d|d["binding"]["part"]==*part).collect();if tracks.is_empty(){return Err(err(format!("Part {part} 没有启用音频，请生成或上传并绑定")));}let generated=tracks.iter().filter(|d|d["origin"]=="book"||d["origin"]=="user").count();if generated>0&&tracks.len()>1{return Err(err(format!("Part {part} 同时启用了合成和其他版本，请选择一个版本")));}}}
 Ok(out)
}
