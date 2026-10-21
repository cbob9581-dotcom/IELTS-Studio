import type {DesktopPort} from "../platform/desktop";
import {errorText} from "../platform/desktop";
import type {CloudJob} from "./types";
import {taskActive} from '../tasks/types';
import {jobTitle} from './JobProgress';
type Snapshot={busy:boolean;job:CloudJob|null;error:string};
const runners=new WeakMap<DesktopPort,TaskRunner>();
class TaskRunner{
 private state:Snapshot={busy:false,job:null,error:""};
 private listeners=new Set<()=>void>();
 private stopped=false;
 private nativeTask:string|null=null;
 constructor(private port:DesktopPort){}
 snapshot=()=>this.state;
 subscribe=(f:()=>void)=>{this.listeners.add(f);return()=>{this.listeners.delete(f);};};
 private emit(change:Partial<Snapshot>){this.state={...this.state,...change};this.listeners.forEach(f=>f());}
 async pause(){this.stopped=true;if(this.nativeTask&&this.port.taskPause){await this.port.taskPause(this.nativeTask);return;}if(this.state.job&&this.state.busy)await this.port.cloudCancel(this.state.job.id);}
 async run(initial:CloudJob):Promise<CloudJob>{
  if(this.state.busy)throw Error("已有任务在后台运行，请先暂停或等待完成。");
  this.stopped=false;this.emit({busy:true,job:initial,error:""});
  if(this.port.taskEnqueue&&this.port.taskRead){
   try{
    const task=await this.port.taskEnqueue({kind:'cloud',owner:initial.id,targets:[initial.id],title:jobTitle(initial)});this.nativeTask=task.id;
    while(!this.stopped){
     const current=await this.port.taskRead(task.id);if(!taskActive(current.state)){if(current.error)this.emit({error:current.error});break;}
     if(this.port.cloudProgress){const progress=await this.port.cloudProgress(initial.id);this.emit({job:{...this.state.job!,...progress}});}
     await new Promise(r=>setTimeout(r,1000));
    }
    const job=await this.port.cloudRead(initial.id);this.emit({job});return job;
   }catch(e){this.emit({error:errorText(e)});throw e;}finally{this.nativeTask=null;this.emit({busy:false});}
  }
  let polling=false,version=0;
  const timer=setInterval(async()=>{if(polling||!this.port.cloudProgress)return;polling=true;const observed=version;
   try{const summary=await this.port.cloudProgress(initial.id);if(version===observed&&this.state.busy&&this.state.job?.id===initial.id)this.emit({job:{...this.state.job,...summary}});}catch{/* The step request owns failures. */}finally{polling=false;}
  },1500);
  let job=initial;
  try{
   while(!this.stopped&&!['complete','review_required','failed','paused','source_required'].includes(job.state)){
    const stage=job.stage;job=await this.port.cloudStep(job.id);version++;this.emit({job});
    if(job.kind==='import'&&job.stage===stage&&stage===1&&job.state==='pending'&&!this.stopped)await new Promise(r=>setTimeout(r,3000));
   }
   return job;
  }catch(e){this.emit({error:errorText(e)});throw e;}
  finally{clearInterval(timer);this.emit({busy:false});}
 }
}
export function taskRunner(port:DesktopPort){let runner=runners.get(port);if(!runner){runner=new TaskRunner(port);runners.set(port,runner);}return runner;}
