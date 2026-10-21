import {TaskPolicyPanel} from "./TaskPolicyPanel";
import {CalibrationPanel} from './CalibrationPanel';
import {useEffect,useState} from 'react';
import type {DesktopPort} from '../platform/desktop';
import {errorText} from '../platform/desktop';
import {taskActive,type BackgroundTask} from './types';
const names:Record<string,string>={queued:'排队中',running:'处理中',pausing:'当前步骤完成后暂停',paused:'已暂停，可继续',ready:'本次步骤完成',failed:'需要处理',cancelled:'已取消'};
export function TaskCenter({port,onOpen}:{port:DesktopPort;onOpen:(task:BackgroundTask)=>void}){
 const [tasks,setTasks]=useState<BackgroundTask[]>([]),[error,setError]=useState('');
 useEffect(()=>{let alive=true,pending=false;const read=async()=>{if(pending||!port.taskList)return;pending=true;try{const rows=await port.taskList();if(alive)setTasks(rows);}catch(e){if(alive)setError(errorText(e));}finally{pending=false;}};void read();const timer=setInterval(read,1500);return()=>{alive=false;clearInterval(timer);};},[port]);
 async function action(task:BackgroundTask,pause:boolean){try{setError('');await(pause?port.taskPause?.(task.id):port.taskResume?.(task.id));if(port.taskList)setTasks(await port.taskList());}catch(e){setError(errorText(e));}}
 if(!port.taskList)return null;
 return <section className="settings-card"><h1>后台任务</h1><TaskPolicyPanel port={port}/><CalibrationPanel port={port}/><p className="muted">切换页面后由桌面后台继续处理。关闭应用会停止本机执行；再次启动后，中断任务保留成功步骤并等待你继续。暂停不会撤回已经提交的云请求。</p>{error&&<p role="alert" className="error">{error}</p>}{!tasks.length&&<p>暂无后台任务。</p>}{tasks.map(t=><article key={t.id} className="settings-card"><h3>{t.title}</h3><p>{names[t.state]??t.state} · {t.completed.length}/{t.targets.length} 个处理单元已完成</p>{t.operation?.label&&<p>{t.operation.label}{t.operation.total?` · ${t.operation.done??0}/${t.operation.total} ${t.operation.unit??''}`:''}</p>}{t.usage&&<p className="muted">文字/评测请求记录：{t.usage.requests} 次 · 已返回用量：输入 {t.usage.inputTokens??'未提供'} / 输出 {t.usage.outputTokens??'未提供'} Token（未确认请求仍占预算）</p>}{t.error&&<p className="error">{t.error}</p>}<div className="library-actions"><button onClick={()=>onOpen(t)}>查看对应内容</button>{taskActive(t.state)&&<button disabled={t.state==='pausing'} onClick={()=>action(t,true)}>暂停</button>}{['paused','failed'].includes(t.state)&&<button onClick={()=>action(t,false)}>继续未完成步骤</button>}{['queued','paused','failed'].includes(t.state)&&port.taskCancel&&<button onClick={async()=>{try{await port.taskCancel!(t.id);if(port.taskList)setTasks(await port.taskList());}catch(e){setError(errorText(e));}}}>取消本次任务</button>}</div></article>)}</section>;
}
