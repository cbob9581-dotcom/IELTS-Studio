import {useSyncExternalStore} from "react";
import type {DesktopPort} from "../platform/desktop";
import {taskRunner} from "./taskRunner";
import {jobTitle} from "./JobProgress";
export function TaskStatus({port}:{port:DesktopPort}){
 const runner=taskRunner(port),s=useSyncExternalStore(runner.subscribe,runner.snapshot);
 if(!s.busy)return null;
 return <div className="notice" role="status">后台任务：{s.job?jobTitle(s.job):"处理中"} · {s.job?.operation?.label??"正在请求云服务"}。切换页面可继续处理。
 <button onClick={()=>runner.pause().catch(()=>{})}>暂停后台任务</button></div>;
}
