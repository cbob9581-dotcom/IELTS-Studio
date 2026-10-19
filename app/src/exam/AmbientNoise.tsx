import {useEffect,useRef,useState} from "react";
import type {AppConfig} from "../configuration/types";
import {noiseEvent,playNoise,type NoiseKind} from "./noise";
export function AmbientNoise({config,section,seed,active,onEvent,sequence=0}:{config:AppConfig;section:string;seed:string;active:boolean;onEvent?:(kind:string)=>void;sequence?:number}){
 const [error,setError]=useState("");
 const sequenceRef=useRef(sequence);sequenceRef.current=sequence;
 useEffect(()=>{
  const allowed=(["handwriting","keyboard","waterPump"] as NoiseKind[]).filter(k=>config.noise[k]);
  if(!active||config.noise.profile==="clean"||!allowed.length||section==="speaking"&&!config.noise.speakingEnabled)return;
  let stopped=false,timer:ReturnType<typeof setTimeout>,index=sequenceRef.current;const context=new AudioContext(),gain=context.createGain();
  gain.gain.value=config.audio.masterVolume*config.noise.volume*(section==="listening"?.3:.6);gain.connect(context.destination);
  const unlock=()=>{context.resume().catch(()=>setError("环境声未能播放，请检查输出设备。"));};
  window.addEventListener("pointerdown",unlock);window.addEventListener("keydown",unlock);
  const next=()=>{if(stopped)return;const e=noiseEvent(seed,index++,allowed,config.noise.profile);if(!e)return;
   timer=setTimeout(()=>{if(context.state==="running"){playNoise(context,gain,e.kind,e.seed);onEvent?.(e.kind);}next();},e.delayMs);
  };
  (async()=>{
   if(config.audio.outputDeviceId){const c=context as AudioContext&{setSinkId?:(id:string)=>Promise<void>};if(!c.setSinkId)throw Error("当前环境声输出不支持所选设备；环境声已停止，请切换系统默认输出。");await c.setSinkId(config.audio.outputDeviceId);}
   await context.resume();if(!stopped)next();
  })().catch(e=>{if(!stopped)setError(e.message??"环境声初始化失败");});
  return()=>{stopped=true;clearTimeout(timer);window.removeEventListener("pointerdown",unlock);window.removeEventListener("keydown",unlock);gain.disconnect();context.close().catch(()=>{});};
 },[config,section,seed,active,onEvent]);
 return error?<div className="game-error" role="status">{error}</div>:null;
}
