import {useEffect,useRef,useState} from "react";
import type {DesktopPort} from "../platform/desktop";
import {errorText} from "../platform/desktop";
export function RecordingPlayback({port,attemptId,takeId,evidence=[]}:{port:DesktopPort|null;attemptId:string;takeId:string;evidence?:string[]}){
 const [url,setUrl]=useState(""),[busy,setBusy]=useState(false),[error,setError]=useState("");
 const seek=useRef(0);
 const audio=useRef<HTMLAudioElement>(null),owned=useRef(""),alive=useRef(true);
 useEffect(()=>{alive.current=true;return()=>{alive.current=false;if(owned.current)URL.revokeObjectURL(owned.current);};},[]);
 async function load(at=0){if(!port?.readSpeakingRecording)return;setError("");setBusy(true);
  try{
   if(owned.current){if(audio.current){audio.current.currentTime=at;await audio.current.play();}return;}
   const bytes=await port.readSpeakingRecording(attemptId,takeId);if(!alive.current)return;
   seek.current=at;owned.current=URL.createObjectURL(new Blob([bytes],{type:"audio/wav"}));setUrl(owned.current);

  }catch(e){if(alive.current)setError(errorText(e));}finally{if(alive.current)setBusy(false);}
 }
 const times=[...new Set(evidence.flatMap(s=>{const matches=s.matchAll(/\b(\d{1,2}):(\d{2})\b/g);return [...matches].map(m=>Number(m[1])*60+Number(m[2]));}))];
 return <div className="recording-playback"><button disabled={busy||!port?.readSpeakingRecording} onClick={()=>load()}>{busy?"正在读取录音…":"回听原始回答"}</button>
  {times.map(t=><button disabled={busy||!port?.readSpeakingRecording} key={t} onClick={()=>load(t)}>定位 {Math.floor(t/60)}:{String(t%60).padStart(2,"0")}</button>)}
  {url&&<audio ref={audio} src={url} controls preload="metadata" onLoadedMetadata={async()=>{const a=audio.current;if(!a)return;
   try{const config=await port?.getConfig?.();if(config){a.volume=config.audio.masterVolume;if(config.audio.outputDeviceId){const player=a as HTMLAudioElement&{setSinkId?:(id:string)=>Promise<void>};if(!player.setSinkId)throw Error("录音播放器不支持所选输出设备");await player.setSinkId(config.audio.outputDeviceId);}}
    if(!alive.current)return;a.currentTime=Math.min(seek.current,a.duration||seek.current);await a.play();
   }catch(e){if(alive.current)setError(errorText(e));}
  }} onError={()=>setError("录音无法播放，请检查保存文件。")}/>}
  {error&&<p role="alert" className="error">{error}</p>}
 </div>;
}
