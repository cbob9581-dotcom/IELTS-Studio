import {useEffect,useRef,useState} from "react";
import type {DesktopPort} from "../platform/desktop";
import {errorText} from "../platform/desktop";
import type {Mode,Section,PaperVersion} from "../domain/paper";
import type {CloudJob} from "./types";
import {taskRunner} from "./taskRunner";
import {DraftMedia} from "./DraftMedia";
import {PaperDraftEditor} from "./PaperDraftEditor";
import {JobProgress,jobTitle,failureAdvice} from "./JobProgress";
const labels:Record<string,string>={pending:"可继续",running:"处理中",failed:"需要处理",paused:"已暂停",review_required:"待核对",complete:"已发布到题库"};
export function CloudLibrary({port,mode,section,part,kind,onPublished}:{port:DesktopPort;mode:Mode;section:Section;part:string;kind:string;onPublished:()=>void}){
 const [jobs,setJobs]=useState<CloudJob[]>([]),[selected,setSelected]=useState<CloudJob|null>(null);
 const [topic,setTopic]=useState(""),[error,setError]=useState(""),[busy,setBusy]=useState(false),[draft,setDraft]=useState("");
 const [confirmed,setConfirmed]=useState(false);
 const [filter,setFilter]=useState("all"),[pageRanges,setPageRanges]=useState(""),[repairOcr,setRepairOcr]=useState(true),[forceOcr,setForceOcr]=useState(false);
 const alive=useRef(true);
 const runner=taskRunner(port);
 const refresh=()=>port.cloudList?.().then(js=>{if(alive.current)setJobs(js.filter(j=>j.kind!=="assessment"&&j.request?.sourceOnly!==true));}).catch(e=>{if(alive.current)setError(errorText(e));});
 const show=(j:CloudJob)=>{setSelected(j);setDraft(j.paper?JSON.stringify(j.paper,null,2):"");setConfirmed(false);};
 useEffect(()=>{alive.current=true;refresh();
  const sync=()=>{const s=runner.snapshot();setBusy(s.busy);if(s.job&&s.job.kind!=="assessment"){
   setSelected(s.job);if(s.job.paper)setDraft(JSON.stringify(s.job.paper,null,2));
   setJobs(js=>[s.job!,...js.filter(j=>j.id!==s.job!.id)]);
  }if(s.error)setError(s.error);};
  if(runner.snapshot().busy)sync();const off=runner.subscribe(sync);
  return()=>{alive.current=false;off();};
 },[port,runner]);
 async function run(job:CloudJob){setError("");show(job);try{await runner.run(job);}catch(e){if(alive.current)setError(errorText(e));}finally{if(alive.current)refresh();}}
 async function create(importFile=false){
  setError("");setBusy(true);
  try{
   const path=importFile?await port.chooseFile(["pdf","doc","docx","ppt","pptx","png","jpg","jpeg","webp","md","txt"]):null;
   if(importFile&&!path)return;
   const request=importFile?{section:mode==="exam"?"all":section,pageRanges,repairOcr,forceOcr}:{section:mode==="exam"?"all":section,...(mode==="training"&&part?{part:Number(part)}:{}),...(mode==="training"&&kind?{kind}:{}),topic};
   const job=await port.cloudCreate(importFile?"import":"generation",request,path);await run(job);
  }catch(e){setError(errorText(e));}finally{setBusy(false);}
 }
 async function save(){if(!selected)return;const paper=JSON.parse(draft) as PaperVersion;show(await port.cloudReview(selected.id,paper));}
 return <section className="settings-card cloud-library">
  <h2>云端出题与资料入库</h2>
  <p className="muted">Academic 官方题型蓝图 · 原创模拟题 · 按步骤保存。以下操作会向你配置的服务发送所选资料并可能计费。待核对内容不会进入考试。</p>
  <label>出题主题（可选）<input aria-label="出题主题（可选）" maxLength={500} value={topic} onChange={e=>setTopic(e.target.value)} placeholder="例如 education, urban development；留空由 AI 选题"/></label>
  <details><summary>资料解析选项</summary><label>页码范围（可选）<input value={pageRanges} maxLength={100} onChange={e=>setPageRanges(e.target.value)} placeholder="例如 1-12,15；留空解析全部"/></label><label className="check-row"><input type="checkbox" checked={repairOcr} onChange={e=>setRepairOcr(e.target.checked)}/>AI 校正 OCR 断字与标点（增加处理时间；纯文本直接整理成卷）</label><label className="check-row"><input type="checkbox" checked={forceOcr} onChange={e=>setForceOcr(e.target.checked)}/>PDF 强制 OCR（仅扫描件或文字识别异常时勾选；图片自动开启）</label></details>
  <div className="library-actions"><button disabled={busy} onClick={()=>create()}>AI {mode==="exam"?"生成整套":"生成此项"}</button><button disabled={busy} onClick={()=>create(true)}>上传资料 · MinerU / 文本清洗</button><button disabled={busy} onClick={refresh}>刷新任务</button></div>
  {error&&<p role="alert" className="error">{error}</p>}
  {busy&&<div className="notice">成功内容会自动保存。<button onClick={()=>{runner.pause().catch(e=>setError(errorText(e)));}}>暂停任务</button></div>}
  <div className="library-actions" aria-label="建卷任务分类">{[["all","全部"],["active","进行中"],["review_required","待核对"],["complete","已发布"],["attention","需处理"]].map(([id,name])=><button key={id} aria-pressed={filter===id} onClick={()=>setFilter(id)}>{name} ({jobs.filter(j=>id==="all"||(id==="active"?["running","pending"].includes(j.state):id==="attention"?["failed","paused"].includes(j.state):j.state===id)).length})</button>)}</div>
  <div className="cloud-job-list">{jobs.filter(j=>filter==="all"||(filter==="active"?["running","pending"].includes(j.state):filter==="attention"?["failed","paused"].includes(j.state):j.state===filter)).map(j=><button disabled={busy} key={j.id} onClick={async()=>{setError("");try{show(await port.cloudRead(j.id));}catch(e){setError(errorText(e));}}}>{j.kind==="import"?"资料解析":"AI 出题"} · {jobTitle(j)} · {labels[j.state]??j.state}<small>{j.createdAt?new Date(j.createdAt).toLocaleString():""}</small></button>)}</div>
  {selected&&<article className="cloud-review">
   <h3>{jobTitle(selected)} · {labels[selected.state]??selected.state}</h3>
   <JobProgress job={selected} busy={busy}/>
   {selected.warning&&<p className="notice">{selected.warning}</p>}
   {selected.state==="complete"&&<p className="notice">发布成功。此卷已进入本地题库，可在上方试卷列表选择进入考场。原始资料与草稿另行保留。</p>}
   {selected.error&&<div className="error" role="alert"><strong>{selected.state==="paused"?"任务已暂停":"当前环节没有完成，试卷尚未发布"}</strong><p>{selected.error}</p><p>{failureAdvice(selected.error)}</p></div>}
   <details><summary>任务记录与保存位置</summary><p>任务编号：{selected.id}</p><p>新任务：资产目录 / cloud-jobs / {selected.kind} / 任务编号；旧任务仍保留原路径。正式试卷：资产目录 / papers / 卷编号 / 版本。学习记录保存在单独的学习目录。</p></details>
   {!["complete","review_required"].includes(selected.state)&&<><p className="muted">继续会复用已保存的步骤和 MinerU 批次。若上次网络请求中断，当前未确认步骤可能再次计费。</p><button disabled={busy} onClick={()=>run({...selected,state:"pending"})}>继续 / 重试未完成步骤</button></>}
   {selected.validation&&<div className="notice">独立 AI 校审：{selected.validation.pass?"通过，仍需人工核对":"发现问题，请核对以下题目"}<ul>{selected.validation.issues.map((v,i)=><li key={i}>{typeof v==="string"?v:typeof v==="object"&&v?`${"questionId" in v?v.questionId:""}：${"reason" in v?v.reason:"请对照原文检查"}`:"请对照原文检查"}</li>)}</ul></div>}
   {selected.extraction&&<div className="notice"><h4>资料提取提醒</h4>{selected.extraction.warnings?.length?<ul>{selected.extraction.warnings.map((s,i)=><li key={i}>{s}</li>)}</ul>:<p>提取完成，未报告额外提醒；仍需核对原文和答案。</p>}</div>}
   {selected.sourcePreview&&<details><summary>原文与清洗文本、修改记录</summary><div className="cloud-source-compare"><pre>{selected.sourcePreview}</pre><pre>{selected.cleanedText}</pre></div><pre>{JSON.stringify(selected.changes,null,2)}</pre></details>}
   {selected.paper&&<details open={selected.state==="review_required"}><summary>试卷草稿与媒体绑定（含答案，仅在主页核对）</summary>
     <DraftMedia job={selected} port={port}/>
     <p className="muted">缺失答案请根据原资料补录。上传录音后勾选本项音频，图片可在题目中选择；特殊题型可展开高级结构编辑。发布前会校验答案、引用与媒体完整性。</p>
     {selected.state==="review_required"&&(()=>{try{return <PaperDraftEditor key={selected.id} paper={JSON.parse(draft)} onChange={p=>{setDraft(JSON.stringify(p,null,2));setConfirmed(false);}}/>;}catch{return <p className="error">高级结构内容不是有效 JSON，请先修正。</p>;}})()}
     <details><summary>高级结构编辑 / 只读卷包预览</summary><textarea className="cloud-json" aria-label="试卷草稿 JSON" value={draft} readOnly={selected.state!=="review_required"} onChange={e=>{setDraft(e.target.value);setConfirmed(false);}}/></details>
     {selected.state==="review_required"&&<><button disabled={busy} onClick={()=>save().catch(e=>setError(errorText(e)))}>保存核对修改</button><button disabled={busy} onClick={async()=>{setBusy(true);try{await save();const path=await port.chooseFile(["wav","mp3","ogg","png","jpg","jpeg","webp"]);if(path)show(await port.cloudAddMedia(selected.id,path));}catch(e){setError(errorText(e));}finally{setBusy(false);}}}>添加音频 / 图片</button>
     <ul>{selected.assets?.map(a=><li key={a.id}>{a.id} · {a.mediaType}</li>)}</ul>
     <label className="check-row"><input type="checkbox" checked={confirmed} onChange={e=>setConfirmed(e.target.checked)}/>已核对原文、答案、图表、听力录音与题目对应关系</label>
     <button disabled={busy||selected.kind!=="generation"||!port.cloudRecheck} onClick={async()=>{setBusy(true);setError("");try{await save();show(await port.cloudRecheck!(selected.id));}catch(e){setError(errorText(e));}finally{setBusy(false);}}}>重新校审当前草稿</button>
     <button className="primary" disabled={busy||!confirmed} onClick={async()=>{setBusy(true);setError("");try{await save();show(await port.cloudPublish(selected.id));onPublished();refresh();}catch(e){setError(errorText(e));}finally{setBusy(false);}}}>发布到本地题库</button></>}
   </details>}
  </article>}
 </section>;
}
