import {useEffect,useRef,useState} from "react";
import type {DesktopPort} from "../platform/desktop";
import {errorText} from "../platform/desktop";
import type {Mode,Section,PaperVersion} from "../domain/paper";
import type {CloudJob} from "./types";
import {taskRunner} from "./taskRunner";
import {DraftMedia} from "./DraftMedia";
import {PaperDraftEditor} from "./PaperDraftEditor";
import {JobProgress,jobTitle,failureAdvice,failureSummary} from "./JobProgress";
const labels:Record<string,string>={pending:"可继续",running:"处理中",failed:"需要处理",paused:"已暂停",review_required:"系统准备中",complete:"已发布到题库",source_required:"在资料库继续"};
export function CloudLibrary({port,mode,section,part,kind,onPublished,onSources,initialJobId}:{port:DesktopPort;mode:Mode;section:Section;part:string;kind:string;onPublished:()=>void;onSources?:(id:string)=>void;initialJobId?:string}){
 const [jobs,setJobs]=useState<CloudJob[]>([]),[selected,setSelected]=useState<CloudJob|null>(null);
 const [topic,setTopic]=useState(""),[error,setError]=useState(""),[busy,setBusy]=useState(false),[draft,setDraft]=useState("");

 const [visualKind,setVisualKind]=useState("table");
 const [authoringOpen,setAuthoringOpen]=useState(false);
 const [filter,setFilter]=useState("all"),[pageRanges,setPageRanges]=useState(""),[repairOcr,setRepairOcr]=useState(true),[forceOcr,setForceOcr]=useState(false);
 const alive=useRef(true);
 const runner=taskRunner(port);
 const refresh=()=>port.cloudList?.().then(js=>{if(alive.current)setJobs(js.filter(j=>j.kind!=="assessment"&&j.request?.sourceOnly!==true));}).catch(e=>{if(alive.current)setError(errorText(e));});
 useEffect(()=>{if(!initialJobId)return;let active=true;void port.cloudRead(initialJobId).then(j=>{if(active)show(j);}).catch(e=>{if(active)setError(errorText(e));});return()=>{active=false;};},[port,initialJobId]);
 const show=(j:CloudJob)=>{setSelected(j);setDraft(j.paper?JSON.stringify(j.paper,null,2):"");setAuthoringOpen(false);};
 useEffect(()=>{alive.current=true;refresh();
  const sync=()=>{const s=runner.snapshot();setBusy(s.busy);if(s.job&&s.job.kind!=="assessment"){
   setSelected(s.job);if(s.job.paper)setDraft(JSON.stringify(s.job.paper,null,2));
   setJobs(js=>[s.job!,...js.filter(j=>j.id!==s.job!.id)]);
  }if(s.error)setError(s.error);};
  if(runner.snapshot().busy)sync();const off=runner.subscribe(sync);
  return()=>{alive.current=false;off();};
 },[port,runner]);
 async function run(job:CloudJob){setError("");show(job);try{const result=await runner.run(job);if(result.state==="complete")onPublished();}catch(e){if(alive.current)setError(errorText(e));}finally{if(alive.current)refresh();}}
 async function create(importFile=false){
  setError("");setBusy(true);
  try{
   const path=importFile?await port.chooseFile(["pdf","doc","docx","ppt","pptx","png","jpg","jpeg","webp","md","txt"]):null;
   if(importFile&&!path)return;
   if(importFile&&path&&port.sourceAction){const project=await port.sourceAction("create_from_file",{path});onSources?.(project.id);setError(`资料已加入真题资料库：${project.title}。按 Test 整理，答案、解析和音频可分别补充。`);return;}
   const request=importFile?{section:mode==="exam"?"all":section,pageRanges,repairOcr,forceOcr}:{section:mode==="exam"?"all":section,...(mode==="training"&&part?{part:Number(part)}:{}),...(mode==="training"&&kind?{kind}:{}),topic,...(mode==="exam"||section==="writing"?{visualKind}:{})};
   const job=await port.cloudCreate(importFile?"import":"generation",request,path);await run(job);
  }catch(e){setError(errorText(e));}finally{setBusy(false);}
 }
 async function save(){if(!selected)return;const paper=JSON.parse(draft) as PaperVersion;show(await port.cloudReview(selected.id,paper));}
 return <section className="settings-card cloud-library">
  <h2>云端出题与资料入库</h2>
  <p className="muted">Academic 官方题型蓝图 · 原创模拟题 · 按步骤保存。出题完成文字、图表和自动检查后会自动加入题库。听力 TTS 需单独点击启动；上传资料按 Test 整理，已解析内容可以复用。</p>
  <label>出题主题（可选）<input aria-label="出题主题（可选）" maxLength={500} value={topic} onChange={e=>setTopic(e.target.value)} placeholder="例如 education, urban development；留空由 AI 选题"/></label>
  {(mode==="exam"||section==="writing")&&<label>写作 Task 1 图表类型<select aria-label="写作图表类型" value={visualKind} onChange={e=>setVisualKind(e.target.value)}>{[["table","表格"],["bar","柱图"],["line","折线图"],["pie","饼图"],["process","流程图"],["map","地图对比"],["combined","组合图"]].map(([id,name])=><option key={id} value={id}>{name}</option>)}</select></label>}
  {!port.sourceAction&&<details><summary>资料解析选项</summary><label>页码范围（可选）<input value={pageRanges} maxLength={100} onChange={e=>setPageRanges(e.target.value)} placeholder="例如 1-12,15；留空解析全部"/></label><label className="check-row"><input type="checkbox" checked={repairOcr} onChange={e=>setRepairOcr(e.target.checked)}/>AI 校正 OCR 断字与标点（增加处理时间；纯文本直接整理成卷）</label><label className="check-row"><input type="checkbox" checked={forceOcr} onChange={e=>setForceOcr(e.target.checked)}/>PDF 强制 OCR（仅扫描件或文字识别异常时勾选；图片自动开启）</label></details>}
  <div className="library-actions"><button disabled={busy} onClick={()=>create()}>AI {mode==="exam"?"生成整套":"生成此项"}</button><button disabled={busy} onClick={()=>create(true)}>上传资料到真题资料库</button><button disabled={busy} onClick={refresh}>刷新任务</button></div>
  {error&&<p role="alert" className="error">{error}</p>}
  {busy&&<div className="notice">成功内容会自动保存。<button onClick={()=>{runner.pause().catch(e=>setError(errorText(e)));}}>暂停任务</button></div>}
  <div className="library-actions" aria-label="建卷任务分类">{[["all","全部"],["active","进行中"],["review_required","待核对"],["complete","已发布"],["attention","需处理"]].map(([id,name])=><button key={id} aria-pressed={filter===id} onClick={()=>setFilter(id)}>{name} ({jobs.filter(j=>id==="all"||(id==="active"?["running","pending"].includes(j.state):id==="attention"?["failed","paused"].includes(j.state):j.state===id)).length})</button>)}</div>
  <div className="cloud-job-list">{jobs.filter(j=>filter==="all"||(filter==="active"?["running","pending"].includes(j.state):filter==="attention"?["failed","paused"].includes(j.state):j.state===filter)).map(j=><button disabled={busy} key={j.id} onClick={async()=>{setError("");try{const loaded=await port.cloudRead(j.id);if(loaded.kind==="generation"&&loaded.state==="review_required"&&loaded.validation?.pass===true&&!loaded.audioPending){await run(loaded);}else show(loaded);}catch(e){setError(errorText(e));}}}>{j.kind==="import"?"资料解析":"AI 出题"} · {jobTitle(j)} · {labels[j.state]??j.state}<small>{j.createdAt?new Date(j.createdAt).toLocaleString():""}</small></button>)}</div>
  {selected&&<article className="cloud-review">
   <h3>{jobTitle(selected)} · {labels[selected.state]??selected.state}</h3>
   <JobProgress job={selected} busy={busy}/>
   {selected.warning&&<p className="notice">{selected.warning}</p>}
   {selected.state==="complete"&&<p className="notice">发布成功。此卷已进入本地题库，可在上方试卷列表选择进入考场。原始资料与草稿另行保留。</p>}
   {selected.error&&<div className="error" role="alert"><strong>{selected.state==="paused"?"任务已暂停":"当前环节没有完成，试卷尚未发布"}</strong><p>{failureSummary(selected.error,selected.operation?.phase)}</p><p>{failureAdvice(selected.error)}</p><details><summary>查看具体失败记录</summary><pre>{selected.error}</pre></details></div>}
   {selected.kind==="generation"&&selected.state==="failed"&&!selected.paper&&selected.operation?.phase!=="audio"&&port.cloudRetryText&&<button disabled={busy} onClick={async()=>{setError("");try{await run(await port.cloudRetryText!(selected.id));}catch(e){setError(errorText(e));}}}>重新整理当前部分（仅文字，会调用出题服务）</button>}
   {selected.kind==="generation"&&selected.state==="failed"&&selected.operation?.phase==="audio"&&port.cloudRetryAudio&&<button disabled={busy} onClick={async()=>{setError("");try{await run(await port.cloudRetryAudio!(selected.id));}catch(e){setError(errorText(e));}}}>明确重新请求结果未知的听力片段（可能再次计费）</button>}
   <details><summary>任务记录与保存位置</summary><p>任务编号：{selected.id}</p><p>新任务：资产目录 / cloud-jobs / {selected.kind} / 任务编号；旧任务仍保留原路径。正式试卷：资产目录 / papers / 卷编号 / 版本。学习记录保存在单独的学习目录。</p></details>
   {selected.kind==="import"&&selected.stage>=2&&port.sourceAction&&<button disabled={busy} onClick={async()=>{setBusy(true);setError("");try{const p=await port.sourceAction!("recover_import",{jobId:selected.id});onSources?.(p.id);if(!onSources)setError(`已转入资料库项目 ${p.title??p.id}；现有解析已复用，请从主页真题资料库打开。`);refresh();}catch(e){setError(errorText(e));}finally{setBusy(false);}}}>在资料库继续（复用已解析内容）</button>}
   {selected.sourceProjectId&&onSources&&<button onClick={()=>onSources(selected.sourceProjectId!)}>打开对应资料项目</button>}
   {!["complete","review_required","source_required"].includes(selected.state)&&<><p className="muted">继续会复用已保存的步骤和 MinerU 批次。若上次网络请求中断，当前未确认步骤可能再次计费。</p><button disabled={busy} onClick={()=>run({...selected,state:"pending"})}>继续 / 重试未完成步骤</button></>}
   {!!selected.listeningScripts?.length&&<details><summary>听力文字与角色（合成前可查看）</summary>{selected.listeningScripts!.map(s=><section key={s.part}><h4>Part {s.part}</h4>{s.turns.map((t,i)=><p key={i}><strong>{t.speaker===2?"旁白":`说话人 ${t.speaker+1}`}：</strong>{t.text}</p>)}</section>)}</details>}
   {selected.state==="failed"&&selected.paper&&selected.kind==="generation"&&["review","content_fix"].includes(selected.operation?.phase??"")&&port.cloudFixText&&<button disabled={busy} onClick={async()=>{setError("");try{await run(await port.cloudFixText!(selected.id));}catch(e){setError(errorText(e));}}}>重新请求文字校审与修复（不调用 TTS）</button>}
   {selected.state==="review_required"&&selected.kind==="generation"&&<div className="notice">
     {selected.validation?.pass!==true&&<><p>本次尚未生成可用试卷，TTS 未启动。已保存进度与内部处理记录，无需考生逐题修改。</p><button disabled={busy||!port.cloudFixText||!!selected.humanEditedAt} onClick={async()=>{setError("");try{await run(await port.cloudFixText!(selected.id));}catch(e){setError(errorText(e));}}}>AI 修复并复查（仅文字）</button><p className="muted">只向出题服务发送当前文字和问题，可能产生文字服务费用，不调用 TTS。人工修改后可使用“重新校审当前草稿”。</p></>}
     {selected.validation?.pass===true&&selected.audioPending&&<><p>题目自动检查已通过。选择已有听力录音，或单独选择合成；上传录音不调用 TTS。</p><button disabled={busy} onClick={async()=>{setBusy(true);setError("");try{const path=await port.chooseFile(["wav","mp3","ogg"]);if(path){const job=await port.cloudAddMedia(selected.id,path);show(job);if(job.state==="complete")onPublished();refresh();}}catch(e){setError(errorText(e));}finally{setBusy(false);}}}>上传本套听力录音（不使用 TTS）</button><button disabled={busy||!port.cloudStartAudio} onClick={async()=>{setError("");try{await run(await port.cloudStartAudio!(selected.id));}catch(e){setError(errorText(e));}}}>单独合成听力（{Math.max(0,(selected.audioSegmentCount??0)-(selected.audioCompleted??0))} 段，会消耗 TTS）</button></>}
   </div>}
   <details className="authoring-tools" open={authoringOpen} onToggle={e=>setAuthoringOpen(e.currentTarget.open)}><summary>资料管理与编辑（可选）</summary>{authoringOpen&&<>
   {!!selected.validation?.advisories?.length&&<details><summary>可选改进建议（不阻止发布）</summary><ul>{selected.validation.advisories.map((a,i)=><li key={i}>{a.questionId}：{a.reason}</li>)}</ul></details>}
   {selected.validation&&<div className="notice">独立 AI 校审：{selected.validation.pass?"通过":"尚未通过，以下为系统处理记录"}<ul>{selected.validation.issues.map((v,i)=><li key={i}>{typeof v==="string"?v:typeof v==="object"&&v?`${"questionId" in v?v.questionId:""}：${"reason" in v?v.reason:"请对照原文检查"}`:"请对照原文检查"}</li>)}</ul></div>}
   {selected.extraction&&<div className="notice"><h4>资料提取提醒</h4>{selected.extraction.warnings?.length?<ul>{selected.extraction.warnings.map((s,i)=><li key={i}>{s}</li>)}</ul>:<p>提取完成，未报告额外提醒；仍需核对原文和答案。</p>}</div>}
   {selected.sourcePreview&&<details><summary>原文与清洗文本、修改记录</summary><div className="cloud-source-compare"><pre>{selected.sourcePreview}</pre><pre>{selected.cleanedText}</pre></div><pre>{JSON.stringify(selected.changes,null,2)}</pre></details>}
   {selected.paper&&<details><summary>试卷草稿与媒体绑定（含答案，仅在主页核对）</summary>
     <DraftMedia job={selected} port={port}/>
     <p className="muted">缺失答案请根据原资料补录。上传录音后勾选本项音频，图片可在题目中选择；特殊题型可展开高级结构编辑。发布前会校验答案、引用与媒体完整性。</p>
     {selected.state==="review_required"&&(()=>{try{return <PaperDraftEditor key={selected.id} paper={JSON.parse(draft)} onChange={p=>{setDraft(JSON.stringify(p,null,2));}}/>;}catch{return <p className="error">高级结构内容不是有效 JSON，请先修正。</p>;}})()}
     <details><summary>高级结构编辑 / 只读卷包预览</summary><textarea className="cloud-json" aria-label="试卷草稿 JSON" value={draft} readOnly={selected.state!=="review_required"} onChange={e=>{setDraft(e.target.value);}}/></details>
     {selected.state==="review_required"&&<><button disabled={busy} onClick={()=>save().catch(e=>setError(errorText(e)))}>保存核对修改</button><button disabled={busy} onClick={async()=>{setBusy(true);try{await save();const path=await port.chooseFile(["wav","mp3","ogg","png","jpg","jpeg","webp"]);if(path)show(await port.cloudAddMedia(selected.id,path));}catch(e){setError(errorText(e));}finally{setBusy(false);}}}>添加音频 / 图片</button>
     <ul>{selected.assets?.map(a=><li key={a.id}>{a.id} · {a.mediaType}</li>)}</ul>
     <button disabled={busy||selected.kind!=="generation"||!port.cloudRecheck} onClick={async()=>{setBusy(true);setError("");try{await save();await run(await port.cloudRecheck!(selected.id));}catch(e){setError(errorText(e));}finally{setBusy(false);}}}>重新校审当前草稿</button>
     <button className="primary" disabled={busy||(selected.kind==="generation"&&(selected.validation?.pass!==true||selected.validation.issues.length>0||!!selected.audioPending))} onClick={async()=>{setBusy(true);setError("");try{await save();show(await port.cloudPublish(selected.id));onPublished();refresh();}catch(e){setError(errorText(e));}finally{setBusy(false);}}}>发布到本地题库</button></>}
   </details>}
   </>}
   </details>
  </article>}
 </section>;
}
