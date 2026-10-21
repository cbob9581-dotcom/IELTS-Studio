import {useEffect,useState} from "react";
import type {CloudJob} from "./types";
export const sectionLabels:Record<string,string>={all:"整套四科",listening:"听力",reading:"阅读",writing:"写作",speaking:"口语"};
export function jobTitle(j:CloudJob){
 const scope=sectionLabels[String(j.request.section)]??"试卷";
 return j.paperTitle??String(j.request.sourceName??`${scope}${j.request.part?` Part ${j.request.part}`:""}${j.request.topic?` · ${j.request.topic}`:" · 综合主题"}`);
}
export function JobProgress({job,busy}:{job:CloudJob;busy:boolean}){
 const [now,setNow]=useState(Date.now());
 useEffect(()=>{if(!busy)return;const id=setInterval(()=>setNow(Date.now()),1000);return()=>clearInterval(id);},[busy]);
 const op=job.operation,total=op?.total??0,done=op?.done??0;
 const stopped=job.state==="failed"||job.state==="paused";
 const label=stopped?job.state==="paused"?"已暂停，已完成内容保留":job.kind==="import"?"资料处理已停止，需要处理下列问题":"文字生成或校验未完成，已停止":op?.label;
 const current=busy&&op?.startedAt?Math.max(0,now-op.startedAt):0;
 const seconds=Math.floor(((job.elapsedMs??0)+current)/1000);
 return <div className="job-progress" aria-live="polite"><strong>{label??({generation:"准备生成题目",import:"准备解析资料",assessment:"准备评测"}[job.kind])}</strong>
 {total>0&&<><p>{done} / {total} {op?.unit} · 当前环节</p><progress aria-label="当前环节进度" value={Math.min(done,total)} max={total}/></>}
 {busy&&total<=0&&<p>等待云服务返回；收到页数后显示实际解析进度。</p>}
 <small>累计处理 {Math.floor(seconds/60)} 分 {seconds%60} 秒{busy?" · 当前请求进行中":""}。各环节分别计数，云服务未提供剩余时间。</small>
 </div>;
}
export function failureAdvice(error:string){
 if(/单卷处理范围|资料过长|分卷导入/.test(error))return "这是整本资料进入单卷流程造成的限制。请点击“在资料库继续”，复用已解析内容按 Test 整理，无需再次上传或 OCR。";
 if(/答案键|答案来源|ANSWER_SOURCE/.test(error))return "解析文本未找到可核对的答案页。可转入资料库核对原页、补充答案文件；不会把推测答案发布成真题答案。";
 if(/Token|截断|输出额度|思考/.test(error))return "在设置中使用 DeepSeek 参考参数（输出上限 16384），再继续此任务；新版已关闭默认思考模式。";
 if(/超时|读取.*响应|连接|网络/.test(error))return "网络请求未完整返回。稍后点击继续，只补未完成部分；最后一笔未确认请求可能重复计费。";
 if(/题目|答案|原文|结构|字段|JSON|字数|schema|is not|Additional/.test(error))return "本次模型输出未通过内容检查，草稿没有发布。请查看下面的具体原因，再继续修复当前部分。";
 return "已保留原文件和成功步骤。按上述原因修改对应设置后继续原任务，无需重新上传或重建。";
}
export function failureSummary(error:string,phase?:string){
 if(phase==="audio")return "听力音频尚未全部完成，已完成的片段会保留。";
 if(/单卷处理范围|资料过长|分卷导入/.test(error))return "整本或多套资料进入了旧单卷流程，现有原件和解析结果已保存。";
 if(/答案键|答案来源|ANSWER_SOURCE/.test(error))return "没能把答案与原资料中的答案页对应起来。可以核对原页或补充答案文件。";
 if(/evidence|引文|答案依据/.test(error))return "部分题目的答案依据与原文不一致，已停止文字环节。";
 if(/计分|题数|编号/.test(error))return "当前部分的题号或分值不完整，可以只重新整理失败的文字部分。";
 if(/JSON|字段|结构|字数约束/.test(error))return "出题服务返回的文字格式不完整，可以只重新整理失败的文字部分。";
 return error.split("。原错误：")[0].slice(0,500);
}
