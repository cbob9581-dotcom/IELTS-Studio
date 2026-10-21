import {useSyncExternalStore,useState,useEffect} from "react";
import type {DesktopPort} from "../platform/desktop";
import {taskRunner} from "./taskRunner";
import {jobTitle} from "./JobProgress";
export function TaskStatus({port,onOpen}:{port:DesktopPort;onOpen?:()=>void}){return port.taskList?<NativeStatus port={port} onOpen={onOpen}/>:<LegacyStatus port={port}/>;}
function NativeStatus({port,onOpen}:{port:DesktopPort;onOpen?:()=>void}){
 const [tasks,setTasks]=useState<import('../tasks/types').BackgroundTask[]>([]);
 useEffect(()=>{let alive=true,pending=false;const refresh=async()=>{if(pending)return;pending=true;try{const rows=await port.taskList!();if(alive)setTasks(rows);}catch{}finally{pending=false;}};void refresh();const timer=setInterval(refresh,1500);return()=>{alive=false;clearInterval(timer);};},[port]);
 const active=tasks.filter(t=>['queued','running','pausing'].includes(t.state)),attention=tasks.filter(t=>['paused','failed'].includes(t.state));
 if(!active.length&&!attention.length)return null;
 return <div className="notice" role="status">后台任务：{active.length} 项进行中，{attention.length} 项待处理。{active[0]?.operation?.label??''}<button onClick={onOpen}>打开任务中心</button></div>;
}
function LegacyStatus({port}:{port:DesktopPort}){
 const runner=taskRunner(port),s=useSyncExternalStore(runner.subscribe,runner.snapshot);
 if(!s.busy)return null;
 return <div className="notice" role="status">后台任务：{s.job?jobTitle(s.job):"处理中"} · {s.job?.operation?.label??"正在请求云服务"}。切换页面可继续处理。
 <button onClick={()=>runner.pause().catch(()=>{})}>暂停后台任务</button></div>;
}
