import {splitScriptTurn} from "./listeningAudio";
import {useEffect,useState} from 'react';
import type {DesktopPort} from '../platform/desktop';
import type {AudioBinding,ScriptDraft,SourceDocument,SourceProject} from './types';
type Action=(name:string,payload?:Record<string,unknown>)=>Promise<SourceProject>;
const stages:Record<string,string>={draft:'待确认',pending:'准备生成',synthesizing:'生成语音',assembling:'组装音轨',complete:'完整音轨可用',ready:'试听可用',failed:'需处理'};
export function ListeningAudioPanel({port,project,busy,action,run,report}:{port:DesktopPort;project:SourceProject;busy:boolean;action:Action;run:(p:SourceProject,ids:string[])=>Promise<unknown>;report:(e:unknown)=>void}){
 const [tab,setTab]=useState('book'),[text,setText]=useState(''),[test,setTest]=useState('Test 1'),[part,setPart]=useState(1),[draftId,setDraftId]=useState(''),[defaults,setDefaults]=useState<string[]>([]);
 useEffect(()=>{void port.getConfig?.().then(c=>setDefaults(c.providers.qwenTts.voice.split('|').map(v=>v.trim()).filter(Boolean))).catch(()=>{});},[port]);
 const perform=(f:()=>Promise<unknown>)=>{void f().catch(report);};
 const scripts=project.units.filter(u=>u.role==='transcript');
 const drafts=project.audioDrafts??[];const draft=drafts.find(d=>d.id===draftId);
 const add=async(payload:Record<string,unknown>)=>{const p=await action('transcript_import',{test,part,...payload});setDraftId(p.audioDrafts?.at(-1)?.id??'');};
 return <section className="settings-card listening-audio-panel"><h3>听力音频</h3>
  <div className="library-actions">{[['book','真题脚本合成'],['user','用户文本合成'],['upload','上传音频']].map(([id,label])=><button key={id} aria-pressed={tab===id} onClick={()=>setTab(id)}>{label}</button>)}</div>
  {tab==='book'&&<><p>从已解析资料的 Audioscripts 提取，保留原文和页码。若分组遗漏，请先对源文件执行“按原书标题重新分组”。</p>
   {!scripts.length&&<p>当前项目尚无听力原文题组。可解析包含 Audioscripts 的书籍，或手动划定听力原文范围。</p>}
   <div className="library-actions">{scripts.map(u=><button key={u.id} disabled={busy} onClick={()=>perform(async()=>{const old=drafts.find(d=>(d as ScriptDraft&{sourceUnitId?:string}).sourceUnitId===u.id);if(old){setDraftId(old.id);return;}const p=await action('transcript_extract',{targetId:u.id});setDraftId(p.audioDrafts?.at(-1)?.id??'');})}>{u.test} Part {u.part}</button>)}</div></>}
  {tab==='user'&&<><p>独立输入听力稿，无需 PDF/OCR。可用 JAN: / PETER: 标记角色，未标记时作为单人独白。保留原稿，朗读稿可逐段修改。</p>
   <label>目标 Test <input value={test} onChange={e=>setTest(e.target.value)}/></label><label>Part <select value={part} onChange={e=>setPart(+e.target.value)}>{[1,2,3,4].map(n=><option key={n}>{n}</option>)}</select></label>
   <textarea aria-label="用户听力文本" rows={8} value={text} onChange={e=>setText(e.target.value)} placeholder="JAN: Hello.&#10;PETER: Good morning."/>
   <div className="library-actions"><button disabled={busy||!text.trim()} onClick={()=>perform(()=>add({text}))}>从文本创建听力稿</button><button disabled={busy} onClick={()=>perform(async()=>{const path=await port.chooseFile(['txt','md']);if(path)await add({path});})}>导入 TXT / MD</button></div></>}
  {tab==='upload'&&<><p>选择一个或多个 MP3、PCM WAV、Vorbis OGG（每文件上限100MB）。支持整套文件、每 Part 文件及同一 Part 多文件。播放顺序以填写的序号为准，上传音频不调用云服务。</p>
   <button disabled={busy} onClick={()=>perform(async()=>{const files=await port.chooseFiles?.(['mp3','wav','ogg'])??[];if(files.length)await action('add_documents',{files:files.map(path=>({path,roles:['audio']}))});})}>选择一个或多个音频文件</button></>}
  {tab!=='upload'&&<><label>已保存听力稿 <select value={draftId} onChange={e=>setDraftId(e.target.value)}><option value="">选择稿件</option>{drafts.map(d=><option key={d.id} value={d.id}>{d.test} Part {d.part} · {d.origin==='book'?'真题':'用户文本'} · {stages[d.state]??d.state}</option>)}</select></label>
   {draft&&<TranscriptEditor key={`${draft.id}-${draft.revision}`} draft={draft} defaults={defaults} busy={busy} action={action} run={run} report={report} port={port} projectId={project.id}/>}</>}
  <details open={tab==='upload'}><summary>音频资产与试卷绑定</summary><p>生成后的音轨先保存为未启用；核对后启用并绑定。相同 Part 的旧版或其他来源须停用，避免重复播放。</p>
   {project.documents.filter(d=>d.state==='media').map(d=><AudioAsset key={`${d.id}-${JSON.stringify(d.binding)}`} doc={d} projectId={project.id} port={port} busy={busy} action={action} report={report}/>)}</details>
 </section>;
}
function TranscriptEditor({draft,defaults,busy,action,run,report,port,projectId}:{draft:ScriptDraft;defaults:string[];busy:boolean;action:Action;run:(p:SourceProject,ids:string[])=>Promise<unknown>;report:(e:unknown)=>void;port:DesktopPort;projectId:string}){
 const [edit,setEdit]=useState(()=>{const d=structuredClone(draft);[...new Set(d.turns.map(t=>t.speaker))].forEach((s,i)=>{if(!d.voices[s])d.voices[s]=defaults[i%Math.max(defaults.length,1)]??'';});return d;});
 const perform=(f:()=>Promise<unknown>)=>{void f().catch(report);};
 const save=()=>action('transcript_save',{targetId:draft.id,draftRevision:draft.revision,draft:edit});
 const generate=async(preview=false)=>{let p=await save();p=await action('audio_start',{revision:p.revision,targetId:draft.id,preview});await run(p,[draft.id]);};
 const speakers=[...new Set(edit.turns.map(t=>t.speaker))];
 useEffect(()=>{if(defaults.length)setEdit(d=>({...d,voices:Object.fromEntries([...new Set(d.turns.map(t=>t.speaker))].map((s,i)=>[s,d.voices[s]||defaults[i%defaults.length]]))}));},[defaults]);
 return <div><h4>{draft.test} Part {draft.part} · {stages[draft.state]??draft.state}</h4>
  <p>正文 {edit.turns.length} 段 · {edit.turns.reduce((n,t)=>n+Array.from(t.text).length,0)} 字符；生成按段计费。自然句内节奏由标点及模型处理，换人和读题停顿由下面的时间线控制。当前接口不提供独立语速设置。</p>
  {draft.job&&<p role="status">语音片段完成 {draft.job.done}/{draft.job.total}；已生成内容保存在本地。暂停或关闭后可以继续。</p>}
  {draft.error&&<p role="alert" className="error">{draft.error}</p>}
  {draft.state==='failed'&&<button disabled={busy} onClick={()=>perform(async()=>{let p=await action('audio_reset_failed',{targetId:draft.id});p=await action('audio_start',{revision:p.revision,targetId:draft.id});await run(p,[draft.id]);})}>重新请求失败片段（下载地址过期时使用，可能计费）</button>}
  <fieldset disabled={busy}><legend>角色与音色</legend>{speakers.map(s=><label key={s}>{s} <input aria-label={`${s} 音色`} value={edit.voices[s]??''} placeholder="例如 Cherry / Aiden" onChange={e=>setEdit({...edit,voices:{...edit.voices,[s]:e.target.value}})}/></label>)}
  <label>语音生成并发数 <select value={edit.parallelism??3} onChange={e=>setEdit({...edit,parallelism:+e.target.value})}>{[1,2,3].map(n=><option key={n} value={n}>{n}</option>)}</select></label><small>云请求不稳定时可选1；修改后保存，再继续生成，会复用已完成片段。</small>
  <label>Test <input value={edit.test} onChange={e=>setEdit({...edit,test:e.target.value})}/></label><label>Part <select value={edit.part} onChange={e=>setEdit({...edit,part:+e.target.value})}>{[1,2,3,4].map(n=><option key={n}>{n}</option>)}</select></label></fieldset>
  <details><summary>原文与来源（保留原稿）</summary><p>来源页：{[...new Set(draft.sourceRefs.map(r=>r.page))].join('、')||'用户提供'}</p><pre style={{whiteSpace:'pre-wrap',maxHeight:300,overflow:'auto'}}>{draft.rawText}</pre></details>
  <details open><summary>朗读稿和节奏时间线</summary><div style={{maxHeight:520,overflowY:'auto'}}>{edit.events.map((event,index)=>event.type==='silence'?<div key={index}><label>停顿（秒） <input disabled={busy} type="number" min={0} max={120} step={0.1} value={event.durationMs/1000} onChange={e=>setEdit({...edit,events:edit.events.map((x,i)=>i===index?{type:'silence',durationMs:Math.round(+e.target.value*1000)}:x)})}/></label><button disabled={busy} onClick={()=>setEdit({...edit,events:edit.events.filter((_,i)=>i!==index)})}>删除停顿</button></div>:<div key={index}>{(()=>{const t=edit.turns.find(t=>t.id===event.turnId)!;return <><label>{t.id} 角色 <input disabled={busy} value={t.speaker} onChange={e=>setEdit({...edit,turns:edit.turns.map(x=>x.id===t.id?{...x,speaker:e.target.value}:x)})}/></label><textarea disabled={busy} aria-label={`${t.id} 正文`} rows={3} value={t.text} onChange={e=>setEdit({...edit,turns:edit.turns.map(x=>x.id===t.id?{...x,text:e.target.value}:x)})}/><small>{Array.from(t.text).length}/600 字符</small><button disabled={busy} onClick={()=>setEdit(splitScriptTurn(edit,t.id))}>按句拆分此段</button><button disabled={busy} onClick={()=>setEdit({...edit,events:[...edit.events.slice(0,index+1),{type:'silence',durationMs:600},...edit.events.slice(index+1)]})}>在此后插入停顿</button></>;})()}</div>)}</div></details>
  <div className="library-actions"><button disabled={busy} onClick={()=>perform(save)}>保存朗读稿</button><button disabled={busy} onClick={()=>perform(()=>generate(true))}>生成前3段试听（云服务）</button><button disabled={busy} onClick={()=>perform(()=>generate(false))}>生成完整 Part</button>{draft.job&&draft.state!=='complete'&&<button disabled={busy} onClick={()=>perform(async()=>{const p=await action('audio_start',{targetId:draft.id});await run(p,[draft.id]);})}>继续未完成片段</button>}</div>
  {(draft as ScriptDraft&{previewAssetId?:string}).previewAssetId&&<AssetPreview port={port} projectId={projectId} docId={(draft as ScriptDraft&{previewAssetId:string}).previewAssetId}/ >}
  {draft.assetId&&<AssetPreview port={port} projectId={projectId} docId={draft.assetId}/ >}
 </div>;
}
function AssetPreview({port,projectId,docId}:{port:DesktopPort;projectId:string;docId:string}){
 const [url,setUrl]=useState(''),[error,setError]=useState('');
 useEffect(()=>{let active=true,object='';void port.sourceAsset?.(projectId,docId,null).then(data=>{if(active){object=URL.createObjectURL(new Blob([data]));setUrl(object);}}).catch(()=>setError('试听加载失败，请重新打开此稿件。'));return()=>{active=false;if(object)URL.revokeObjectURL(object);};},[port,projectId,docId]);
 return url?<audio aria-label="音频试听" controls src={url}/>:<p>{error||'加载试听…'}</p>;
}
function AudioAsset({doc,projectId,port,busy,action,report}:{doc:SourceDocument;projectId:string;port:DesktopPort;busy:boolean;action:Action;report:(e:unknown)=>void}){
 const [binding,setBinding]=useState<AudioBinding>(doc.binding??{test:'Test 1',scope:'part',part:1,order:100,active:false}),[preview,setPreview]=useState(false);
 const [ranges,setRanges]=useState(()=>doc.binding?.segments??[]);
 const perform=(f:()=>Promise<unknown>)=>{void f().catch(report);};
 return <article className="source-document"><strong>{doc.name}</strong><span>{doc.origin==='book'?'真题脚本合成':doc.origin==='user'?'用户文本合成':'上传音频'} · {doc.durationMs?`${(doc.durationMs/1000).toFixed(1)}秒`:'未检测时长'} · {doc.binding?.active===false?'未启用':doc.binding?'已绑定':'未绑定'}</span>
  <button disabled={busy} onClick={()=>perform(()=>action('audio_verify',{targetId:doc.id}))}>{doc.audioVerified?'重新检查可播放性':'检查音频可播放性'}</button><button onClick={()=>setPreview(!preview)}>试听 / 收起</button>{preview&&<AssetPreview port={port} projectId={projectId} docId={doc.id}/>}
  {!(doc as SourceDocument&{preview?:boolean}).preview&&<fieldset disabled={busy}><legend>试卷播放绑定</legend><label>Test <input value={binding.test} onChange={e=>setBinding({...binding,test:e.target.value})}/></label><label>范围 <select value={binding.scope==='test'?'test':binding.part??1} onChange={e=>setBinding({...binding,scope:e.target.value==='test'?'test':'part',part:e.target.value==='test'?undefined:+e.target.value})}><option value="test">整套听力</option>{[1,2,3,4].map(n=><option value={n} key={n}>Part {n}</option>)}</select></label><label>播放序号 <input type="number" min={0} max={10000} value={binding.order??0} onChange={e=>setBinding({...binding,order:+e.target.value})}/></label><label><input type="checkbox" checked={binding.active!==false} onChange={e=>setBinding({...binding,active:e.target.checked})}/>启用此音频</label>
  {binding.scope==='test'&&<details><summary>标记 Part 起止时间（分项训练需要）</summary>{[1,2,3,4].map(part=>{const value=ranges.find(r=>r.part===part);return <div key={part}><label><input type="checkbox" checked={!!value} onChange={e=>setRanges(e.target.checked?[...ranges,{part,startMs:0,endMs:1000}]:ranges.filter(r=>r.part!==part))}/>Part {part}</label>{value&&<>{(['startMs','endMs'] as const).map(k=><label key={k}>{k==='startMs'?'开始':'结束'}（秒）<input type="number" min={0} step={0.1} value={value[k]/1000} onChange={e=>setRanges(ranges.map(r=>r.part===part?{...r,[k]:Math.round(+e.target.value*1000)}:r))}/></label>)}</>}</div>;})}</details>}
  <button disabled={!doc.audioVerified} onClick={()=>perform(()=>action('bind_audio',{targetId:doc.id,...binding,segments:binding.scope==='test'?[...ranges].sort((a,b)=>a.startMs-b.startMs):[]}))}>保存音频绑定</button></fieldset>}
 </article>;
}
