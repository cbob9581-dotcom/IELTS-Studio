import {RecordingPlayback} from "../speaking/RecordingPlayback";
import {useEffect,useRef,useState} from "react";
import type {DesktopPort} from "../platform/desktop";
import {errorText} from "../platform/desktop";
import type {CloudJob} from "./types";
const criteriaLabels:Record<string,string>={task:"任务完成 / 任务回应",coherence:"连贯与衔接",fluency:"流利度与连贯性",lexical:"词汇资源",grammar:"语法范围与准确性",pronunciation:"发音"};
export function AssessmentPanel({port,attemptId}:{port:DesktopPort|null;attemptId:string}){
 const [job,setJob]=useState<CloudJob|null>(null),[error,setError]=useState(""),[busy,setBusy]=useState(false);
 const alive=useRef(true),stopped=useRef(false);
 useEffect(()=>{alive.current=true;port?.cloudList?.().then(async js=>{const found=js.find(j=>j.kind==="assessment"&&j.request.attemptId===attemptId);const full=found?await port.cloudRead(found.id):null;if(alive.current)setJob(full);}).catch(e=>{if(alive.current)setError(errorText(e));});return()=>{alive.current=false;stopped.current=true;};},[port,attemptId]);
 async function run(){if(!port)return;setBusy(true);setError("");stopped.current=false;
  try{let j=job??await port.cloudCreate("assessment",{attemptId},null);if(alive.current)setJob(j);
   while(alive.current&&!stopped.current&&j.state!=="complete") {j=await port.cloudStep(j.id);if(alive.current)setJob(j);if(["failed","paused","review_required"].includes(j.state))break;}
  }catch(e){if(alive.current)setError(errorText(e));}finally{if(alive.current)setBusy(false);}
 }
 return <section className="settings-card"><h2>云端评测与改进建议</h2>
 <p className="muted">写作按公开四项标准评估；口语向音频模型发送实际录音及原始识别文本。分数为未校准 AI 练习参考，不等同于官方 IELTS 成绩。开始评测会调用已配置云服务。</p>
 {job?.report&&<div className="report-score"><span>{job.report.label}</span><strong>{job.report.estimatedBand.toFixed(1)}</strong><p>{job.report.aggregation}</p></div>}
 {job?.state!=="complete"&&<button disabled={!port||busy} onClick={run}>{busy?"评测中…":job?"继续 / 重试评测":"开始云端评测"}</button>}
 {busy&&<button onClick={()=>{stopped.current=true;if(job)port?.cloudCancel(job.id).catch(e=>setError(errorText(e)));}}>停止后续评测</button>}
 {(error||job?.error)&&<p className="error" role="alert">{error||job?.error}</p>}
 {job?.evaluations?.map((e,i)=><article className="settings-card" key={e.turnId??e.task??i}><h3>{e.task?`Task ${e.task}`:`口语轮次 ${i+1}`} · {e.model}</h3><p>{e.result.summary}</p>
  {e.result.criteria.map(c=><div key={c.name}><h4>{criteriaLabels[c.name]??c.name} · {c.band.toFixed(1)}</h4><p>{c.feedback}</p><blockquote>{c.evidence.join("；")}</blockquote></div>)}
  <>{e.recordingId&&<RecordingPlayback port={port} attemptId={attemptId} takeId={e.recordingId} evidence={e.result.criteria.flatMap(c=>c.evidence)}/>}</><ul>{e.result.improvements?.map((x,i)=><li key={i}>{x}</li>)}</ul><small>{e.recordingId?`原始录音 ${e.recordingId} · ${e.durationSeconds?.toFixed(1)} 秒`:`作答 ${e.wordCount} 词`}</small>
 </article>)}
 </section>;
}
