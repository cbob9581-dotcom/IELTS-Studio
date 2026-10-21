import {useEffect,useState} from 'react';
import type {DesktopPort} from '../platform/desktop';
import {errorText} from '../platform/desktop';
export type TaskPolicy={backgroundConcurrency:number;ocrConcurrency:number;ttsConcurrency:number;maxTextRequests:number;maxContentFixRounds:number};
export function TaskPolicyPanel({port}:{port:DesktopPort}){
 const [policy,setPolicy]=useState<TaskPolicy|null>(null),[error,setError]=useState(''),[message,setMessage]=useState('');
 useEffect(()=>{let alive=true;void port.taskPolicyRead?.().then(p=>{if(alive)setPolicy({...p,maxContentFixRounds:p.maxContentFixRounds??2});}).catch(e=>{if(alive)setError(errorText(e));});return()=>{alive=false;};},[port]);
 if(!policy)return error?<p role="alert" className="error">{error}</p>:null;
 return <details className="settings-card"><summary>任务并发与请求预算</summary><p>预算统计文字生成、结构提取及音频评测请求次数。OCR 页数和 TTS 字符另计；次数与 Token 不是供应商账单金额。修改前请等待运行任务完成或暂停。</p>{([['backgroundConcurrency','后台并行任务',4],['ocrConcurrency','每份资料并行 OCR 批次',2],['ttsConcurrency','每个任务并行 TTS 片段',3],['maxTextRequests','每个任务最多文字/评测请求',1000],['maxContentFixRounds','内容问题最多修复轮数',3]] as const).map(([key,label,max])=><label key={key}>{label}<input type="number" min={1} max={max} value={policy[key]} onChange={e=>setPolicy({...policy,[key]:Number(e.target.value)})}/></label>)}<button onClick={async()=>{try{setError('');setMessage('');await port.taskPolicySave?.(policy);setMessage('任务策略已保存在本地。继续任务时使用新的预算。');}catch(e){setError(errorText(e));}}}>保存任务策略</button>{message&&<p role="status">{message}</p>}{error&&<p role="alert" className="error">{error}</p>}</details>;
}
