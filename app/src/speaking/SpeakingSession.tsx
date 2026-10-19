import { useCallback,useEffect,useRef,useState } from "react";
import type { AppConfig } from "../configuration/types";
import type { PaperVersion } from "../domain/paper";
import type { AttemptSnapshot } from "../domain/session";
import type { DesktopPort } from "../platform/desktop";
import { errorText } from "../platform/desktop";
import { SpeakingRoom } from "../rooms/SpeakingRoom";
import { emptyNotes } from "../rooms/paperSurfaces";
import { createSpeakingState,reduceSpeaking,examinerText,type SpeakingEvent } from "./flow";
import { Microphone,base64Wave,playSpeech,splitUtterance } from "./audio";

export function SpeakingSession({port,paper,attempt,config,onChange,onComplete,interactive,registerBeforeExit}:{
 port:DesktopPort;paper:PaperVersion;attempt:AttemptSnapshot;config:AppConfig;
 onChange:(attempt:AttemptSnapshot)=>void;onComplete:()=>Promise<void>;interactive:boolean;
 registerBeforeExit?:(save:()=>Promise<void>)=>()=>void;
}){
 const current=useRef({attempt,onChange,onComplete});current.current={attempt,onChange,onComplete};
 const [sceneReady,setSceneReady]=useState(false),[ready,setReady]=useState(false);
 const [error,setError]=useState(""),[status,setStatus]=useState("正在准备考场…");
 const [retry,setRetry]=useState(0),[now,setNow]=useState(Date.now());
 const mouthLevel=useRef(0),microphone=useRef<Microphone|null>(null),completed=useRef(false);
 const blankNotes=useRef(emptyNotes());
 const writes=useRef<Promise<void>>(Promise.resolve());
 const saveActive=useRef<()=>Promise<void>>(async()=>{});
 useEffect(()=>registerBeforeExit?.(()=>saveActive.current()),[registerBeforeExit]);
 const pending=useRef<{turnId:string;takeId:string;wav:ArrayBuffer;voiced:boolean}|null>(null);
 const emit=useCallback((event:SpeakingEvent)=>{
   const {attempt:a,onChange:change}=current.current;
   if(!a.speaking||a.status==="submitted")return;
   const speaking=reduceSpeaking(a.speaking,event);
   if(speaking!==a.speaking){const next={...a,speaking};current.current={...current.current,attempt:next};change(next);}
 },[]);
 const sceneLoaded=useCallback(()=>setSceneReady(true),[]);
 useEffect(()=>{
   if(!sceneReady||attempt.status==="submitted")return;
   let disposed=false,owned:Microphone|undefined;
   setReady(false);setError("");
   (async()=>{
     const credentials=await port.credentialStatus();
     for(const [id,name] of [["deepseek","DeepSeek"],["qwenAsr","Qwen ASR"],["qwenTts","Qwen TTS"]] as const){
       const p=config.providers[id];
       if(!p.enabled||!p.model||!p.baseUrl||!credentials[id])throw Error(`请回主页设置 ${name} 的地址、模型、密钥并启用服务。当前不运行静音考试。`);
     }
     if(config.providers.qwenAsr.baseUrl.startsWith("wss:")||config.providers.qwenAsr.model.includes("realtime"))throw Error("本轮适配的是 Qwen ASR 兼容模式 REST 接口，请配置非实时 ASR 型号及 HTTPS 地址。");
     if(!config.providers.qwenTts.voice)throw Error("请先在主页为 Qwen TTS 选择英文女声音色。");
     setStatus("正在连接麦克风…");
     owned=await Microphone.open(config.audio.inputDeviceId,config.audio.sampleRate);
     if(disposed){await owned.dispose();return;}
     microphone.current=owned;
     const {attempt:a,onChange:change}=current.current;
     if(!a.speaking){
       const questions=paper.sections.find(s=>s.section==="speaking")!.questions;
       change({...a,status:"answering",speaking:createSpeakingState(questions,a.scope,Date.now())});
     }
     setReady(true);setStatus("考官正在准备提问…");
   })().catch(e=>{if(!disposed)setError(errorText(e));});
   return ()=>{disposed=true;setReady(false);microphone.current=null;saveActive.current().then(()=>owned?.dispose()).catch(()=>{});};
 },[sceneReady,port,config,retry,paper,attempt.status==="submitted"]);
 useEffect(()=>{
   const timer=setInterval(()=>{const time=Date.now();setNow(time);emit({type:"tick",now:time});},250);
   return ()=>clearInterval(timer);
 },[emit]);
 const flow=attempt.speaking;
 useEffect(()=>{
   if(!ready||!flow||attempt.status==="submitted")return;
   const controller=new AbortController(),ids=new Set<string>();let disposed=false;
   const request=async<T,>(call:(id:string)=>Promise<T>):Promise<T>=>{
     controller.signal.throwIfAborted();const id=crypto.randomUUID();ids.add(id);
     try{const result=await call(id);controller.signal.throwIfAborted();return result;}finally{ids.delete(id);}
   };
   const remember=(capture:AttemptSnapshot["speakingCapture"])=>{
     const c=current.current;const next={...c.attempt,speakingCapture:capture};current.current={...c,attempt:next};c.onChange(next);
   };
   const saveFull=async(capture:NonNullable<typeof pending.current>)=>{
     await writes.current;
     await port.saveSpeakingRecording(attempt.id,capture.takeId,base64Wave(capture.wav));
     const rate=new DataView(capture.wav).getUint32(24,true);
     remember({turnId:capture.turnId,takeId:capture.takeId,chunks:current.current.attempt.speakingCapture?.chunks??[],finalId:capture.takeId,voiced:capture.voiced,durationMs:(capture.wav.byteLength-44)/(rate*2)*1000});
   };
   const recognize=async(takeId:string,voiced:boolean,durationMs:number)=>{
     if(!voiced)throw Error("未检测到足够的有效语音。录音已保存，请检查麦克风后重试此轮。");
     setStatus("正在识别已保存的回答…");
     const text=await request(id=>port.recognizeSpeech(attempt.id,takeId,id));
     if(!text.trim())throw Error("云 ASR 返回空结果。原始录音已保存，可重试识别。");
     pending.current=null;remember(undefined);
     emit({type:"answer",turnId:flow.turnId,text,recordingId:takeId,now:Date.now(),durationMs});
   };
   const acceptRecording=async(capture:NonNullable<typeof pending.current>)=>{
     setStatus("正在保存原始录音…");await saveFull(capture);
     await recognize(capture.takeId,capture.voiced,current.current.attempt.speakingCapture!.durationMs??0);
   };
   const suspend=async()=>{
     const audio=microphone.current?.suspend();
     if(audio){const c=current.current.attempt.speakingCapture;if(c&&c.turnId===flow.turnId){
       const capture={turnId:flow.turnId,takeId:c.takeId,wav:audio.wav,voiced:audio.voiced};pending.current=capture;await saveFull(capture);
     }}
     await writes.current;
   };
   saveActive.current=suspend;
   (async()=>{
     setError("");
     const script=examinerText(flow);
     if(script){
       let text=script;
       if(flow.stage==="question"||flow.stage==="roundoff_question"){
         if(flow.utterance)text=flow.utterance;
         else{
           setStatus("考官正在组织提问…");
           const previous=flow.turns.at(-1);
           const q=flow.questions[flow.index];
           text=await request(id=>port.examinerQuestion(q.prompt,previous?.text??"",q.part,id));
           emit({type:"utterance",turnId:flow.turnId,text,now:Date.now()});
         }
       }
       const chunks=splitUtterance(text);
       let spokenMs=0;
       for(let i=0;i<chunks.length;i++){
         setStatus("正在准备考官语音…");
         const bytes=await request(id=>port.synthesizeSpeech(attempt.id,`${flow.turnId}-${i}`,chunks[i],id));
         controller.signal.throwIfAborted();setStatus("考官正在说话…");
         const spokenAt=performance.now();await playSpeech(bytes,config,controller.signal,mouthLevel);spokenMs+=performance.now()-spokenAt;
       }
       emit({type:"audio_ended",turnId:flow.turnId,now:Date.now(),durationMs:spokenMs});
     }else if(flow.stage==="answering"||flow.stage==="long_turn"||flow.stage==="roundoff_answer"){
       if(pending.current?.turnId===flow.turnId&&pending.current.voiced){await acceptRecording(pending.current);return;}
       const saved=current.current.attempt.speakingCapture;
       if(saved?.turnId===flow.turnId&&saved.finalId&&saved.voiced){await recognize(saved.finalId,true,saved.durationMs??0);return;}
       if(saved?.turnId===flow.turnId&&!saved.finalId&&saved.chunks.length&&port.recoverSpeakingRecording){
         const recovered=await port.recoverSpeakingRecording(attempt.id,`${saved.takeId}-recovered`,saved.chunks);
         remember({...saved,finalId:recovered.takeId,voiced:recovered.voiced,durationMs:recovered.durationMs});
         await recognize(recovered.takeId,recovered.voiced,recovered.durationMs);return;
       }
       const takeId=`${flow.turnId}-${crypto.randomUUID()}`;
       remember({turnId:flow.turnId,takeId,chunks:[]});
       setStatus(flow.stage==="long_turn"?"正在聆听你的陈述…":"正在聆听你的回答…");
       microphone.current!.begin({long:flow.stage==="long_turn",deadline:saved?.finalId&&saved.voiced===false?Date.now()+(flow.stage==="long_turn"?120000:90000):Math.max(Date.now()+1000,flow.deadline??Date.now()+90000),
         onCheckpoint:(wav,index)=>{
           const id=`${takeId}-chunk-${index}`;
           writes.current=writes.current.catch(()=>{}).then(async()=>{
             await port.saveSpeakingRecording(attempt.id,id,base64Wave(wav));
             const saved=current.current.attempt.speakingCapture;
             if(saved?.takeId===takeId)remember({...saved,chunks:[...saved.chunks,id]});
           });
           writes.current.catch(e=>{if(!disposed)setError(`录音分段保存失败：${errorText(e)}`);});
         },
         onDone:(wav,voiced)=>{
           if(disposed)return;const capture={turnId:flow.turnId,takeId,wav,voiced};pending.current=capture;
           acceptRecording(capture).catch(e=>{if(!disposed)setError(errorText(e));});
         }});
     }else if(flow.stage==="preparation")setStatus("准备时间 · 可低头读卡并在草稿纸记笔记");
     else if(flow.stage==="finished"&&!completed.current){setStatus("考试结束，正在提交…");await current.current.onComplete();completed.current=true;}
   })().catch(e=>{if(!disposed&&!(e instanceof DOMException&&e.name==="AbortError"))setError(errorText(e));});
   return ()=>{disposed=true;controller.abort();ids.forEach(id=>port.cancelSpeech(id).catch(()=>{}));suspend().catch(e=>{if(!disposed)setError(errorText(e));});mouthLevel.current=0;};
 // Each turn owns its async requests. Notes, ticks and saving must not replay its speech.
 },[ready,flow?.turnId,retry,port,config,emit,attempt.status==="submitted"]);
 const topic=paper.sections.find(s=>s.section==="speaking")!.questions.find(q=>q.part===2)?.prompt??"";
 const prep=flow?.stage==="preparation"&&flow.deadline?Math.min(60,Math.max(0,Math.ceil((flow.deadline-now)/1000))):null;
 return <>
   <SpeakingRoom port={port} flow={flow} topic={topic} notes={attempt.speakingNotes??blankNotes.current}
     onNotes={notes=>{const c=current.current;const next={...c.attempt,speakingNotes:notes};current.current={...c,attempt:next};c.onChange(next);}}
     onPropsReady={()=>emit({type:"props_ready",now:Date.now()})} onReady={sceneLoaded}
     interactive={interactive} mouthLevel={mouthLevel} renderScale={config.blender.renderScale} assetKey={config.blender.runtimeAssetPath}/>
   {attempt.status!=="submitted"&&<div className={`speaking-status ${error?"error":""}`} role="status">
     {error||status}{prep!==null&&` · ${prep}s`}
     {error&&<button onClick={()=>{setError("");setRetry(x=>x+1);}}>重试当前环节</button>}
   </div>}
 </>;
}
