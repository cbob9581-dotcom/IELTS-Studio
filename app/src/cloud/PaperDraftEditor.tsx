import {useState} from "react";
import type {PaperVersion,Question,ResponseValue} from "../domain/paper";
import {sectionLabels} from "../pages/Home";
import {questionLabels} from "../exam/questionRegistry";
export function PaperDraftEditor({paper,onChange}:{paper:PaperVersion;onChange:(paper:PaperVersion)=>void}){
 const [si,setSi]=useState(0),[qi,setQi]=useState(0);
 const s=paper.sections?.[si],q=s?.questions?.[qi];
 if(!s||!q||!Array.isArray(paper.sections)||!Array.isArray(s.questions))return <p className="notice">草稿结构尚不完整，请在高级结构编辑中核对，或重新提取。</p>;
 const accepted=Array.isArray(q.answer?.accepted)?q.answer.accepted:[];
 const evidenceItems=Array.isArray(q.evidence)?q.evidence.filter(e=>typeof e==="string"):[];
 const assets=Array.isArray(paper.assets)?paper.assets:[];
 const paragraphs=Array.isArray(s.paragraphs)?s.paragraphs:[];
 const edit=(change:Partial<Question>)=>{const p=structuredClone(paper);p.sections[si].questions[qi]={...q,...change};onChange(p);};
 const answer=(value:ResponseValue[])=>edit({answer:{...(q.answer??{caseSensitive:false}),accepted:value}});
 const mapAnswer=accepted[0]&&typeof accepted[0]==="object"&&!Array.isArray(accepted[0])?accepted[0] as Record<string,string>:{};
 const setEvidence=(plain:string,quote:string)=>edit({evidence:[...plain.split("\n").map(s=>s.trim()).filter(Boolean),...quote.split("\n").map(s=>s.trim()).filter(Boolean).map(s=>`ANSWER_SOURCE: ${s}`)]});
 const evidence=evidenceItems.filter(s=>!s.startsWith("ANSWER_SOURCE: ")).join("\n"),source=evidenceItems.filter(s=>s.startsWith("ANSWER_SOURCE: ")).map(s=>s.slice(15)).join("\n");
 const subjective=["writing_task","speaking_prompt"].includes(q.kind);
 return <div className="paper-draft-form">
  <label>试卷名称<input value={paper.title} onChange={e=>onChange({...paper,title:e.target.value})}/></label>
  <div className="filter-row"><label>核对科目<select value={si} onChange={e=>{setSi(Number(e.target.value));setQi(0);}}>{paper.sections.map((s,i)=><option value={i} key={s.section}>{sectionLabels[s.section]}</option>)}</select></label>
  <label>核对题目<select value={qi} onChange={e=>setQi(Number(e.target.value))}>{s.questions.map((q,i)=><option value={i} key={q.id}>第 {q.number} 题 · Part {q.part} · {questionLabels[q.kind]}</option>)}</select></label></div>
  <label>题干<textarea aria-label="题干" value={q.prompt} onChange={e=>edit({prompt:e.target.value})}/></label>
  {Array.isArray(q.options)&&<label>选项（每行一项，答案使用完整选项）<textarea value={q.options.join("\n")} onChange={e=>edit({options:e.target.value.split("\n").filter(Boolean)})}/></label>}
  {!subjective&&<fieldset><legend>标准答案 · 请对照原资料核实</legend>
   {q.kind==="multiple_choice"?<div className="answer-options">{q.options?.map(o=><label key={o}><input type="checkbox" checked={Array.isArray(accepted[0])&&accepted[0].includes(o)} onChange={e=>{const old=Array.isArray(accepted[0])?accepted[0]:[];answer([e.target.checked?[...old,o]:old.filter(x=>x!==o)]);}}/>{o}</label>)}</div>
    :["matching","diagram_label"].includes(q.kind)?(q.kind==="matching"?q.slots:q.labels?.map(l=>l.id))?.map(slot=><label key={slot}>{slot}<input value={mapAnswer[slot]??""} onChange={e=>answer([{...mapAnswer,[slot]:e.target.value}])}/></label>)
    :<label>可接受答案（每行一种写法）<textarea value={accepted.filter((a):a is string=>typeof a==="string").join("\n")??""} onChange={e=>answer(e.target.value.split("\n").map(s=>s.trim()).filter(Boolean))}/></label>}
   {paper.provenance?.kind==="imported"&&<label>原文答案键摘录（须与原资料中的文字完全相同）<textarea value={source} onChange={e=>setEvidence(evidence,e.target.value)}/></label>}
   <label>答案依据（每行一条原文引文）<textarea value={evidence} onChange={e=>setEvidence(e.target.value,source)}/></label>
   <div className="filter-row"><label>最多单词数（仅数字填 0）<input type="number" min={0} value={q.answer?.maxWords??""} onChange={e=>edit({answer:{...(q.answer??{accepted:[],caseSensitive:false}),maxWords:e.target.value?Number(e.target.value):undefined}})}/></label>
   <label>最多数字数<input type="number" min={0} value={q.answer?.maxNumbers??""} onChange={e=>edit({answer:{...(q.answer??{accepted:[],caseSensitive:false}),maxNumbers:e.target.value?Number(e.target.value):undefined}})}/></label></div>
  </fieldset>}
  <label>对应图片<select value={q.imageAssetId??""} onChange={e=>edit({imageAssetId:e.target.value||undefined})}><option value="">无图片</option>{assets.filter(a=>a.mediaType.startsWith("image/")).map(a=><option value={a.id} key={a.id}>{a.path}</option>)}</select></label>
  {s.section==="listening"&&<fieldset><legend>本项听力录音（按列表顺序播放）</legend>{assets.filter(a=>a.mediaType.startsWith("audio/")).map(a=><label className="check-row" key={a.id}><input type="checkbox" checked={s.audioAssetIds?.includes(a.id)??false} onChange={e=>{const p=structuredClone(paper);p.sections[si].audioAssetIds=e.target.checked?[...(s.audioAssetIds??[]),a.id]:(s.audioAssetIds??[]).filter(x=>x!==a.id);if(!e.target.checked&&p.sections[si].audioParts)delete p.sections[si].audioParts![a.id];onChange(p);}}/>{a.path}{s.audioAssetIds?.includes(a.id)&&<select aria-label={`${a.id} 所属 Part`} value={s.audioParts?.[a.id]??""} onChange={e=>{const p=structuredClone(paper);p.sections[si].audioParts={...p.sections[si].audioParts,[a.id]:Number(e.target.value)};onChange(p);}}><option value="" disabled>选择所属 Part</option>{[1,2,3,4].map(n=><option key={n} value={n}>Part {n}</option>)}</select>}</label>)}</fieldset>}
  {paragraphs.length>0&&<details><summary>核对原文段落</summary>{paragraphs.map((paragraph,i)=><label key={paragraph.id}>Part {paragraph.part} · 段落 {i+1}<textarea value={paragraph.text} onChange={e=>{const p=structuredClone(paper);p.sections[si].paragraphs[i].text=e.target.value;onChange(p);}}/></label>)}</details>}
 </div>;
}
