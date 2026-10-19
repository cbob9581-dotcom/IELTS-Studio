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
 const current=busy&&op?.startedAt?Math.max(0,now-op.startedAt):0;
 const seconds=Math.floor(((job.elapsedMs??0)+current)/1000);
 return <div className="job-progress" aria-live="polite"><strong>{op?.label??({generation:"准备生成题目",import:"准备解析资料",assessment:"准备评测"}[job.kind])}</strong>
 {total>0&&<><p>{done} / {total} {op?.unit} · 当前环节</p><progress aria-label="当前环节进度" value={Math.min(done,total)} max={total}/></>}
 {busy&&total<=0&&<p>等待云服务返回；收到页数后显示实际解析进度。</p>}
 <small>累计处理 {Math.floor(seconds/60)} 分 {seconds%60} 秒{busy?" · 当前请求进行中":""}。各环节分别计数，云服务未提供剩余时间。</small>
 </div>;
}
export function failureAdvice(error:string){
 if(/Token|截断|输出额度|思考/.test(error))return "在设置中使用 DeepSeek 参考参数（输出上限 16384），再继续此任务；新版已关闭默认思考模式。";
 if(/超时|读取.*响应|连接|网络/.test(error))return "网络请求未完整返回。稍后点击继续，只补未完成部分；最后一笔未确认请求可能重复计费。";
 if(/题目|答案|原文|结构|字段|JSON|字数|schema|is not|Additional/.test(error))return "本次模型输出未通过内容检查，草稿没有发布。请查看下面的具体原因，再继续修复当前部分。";
 return "已保留原文件和成功步骤。按上述原因修改对应设置后继续原任务，无需重新上传或重建。";
}
