import {useEffect,useRef,useState} from "react";
export type ListeningPosition={track:number;seconds:number;finished:boolean;started:boolean};
export function ListeningPlayer({urls,ranges=[],mode,paused,onReady,onError,initialVolume=.75,outputDeviceId="",position,onPosition}:{
 urls:string[];mode:"exam"|"training";paused:boolean;onReady:()=>void;onError:(s:string)=>void;
 ranges?:{startMs?:number;endMs?:number}[];
 initialVolume?:number;outputDeviceId?:string;position?:ListeningPosition;onPosition?:(p:ListeningPosition)=>void;
}){
 const audio=useRef<HTMLAudioElement>(null);
 const [track,setTrack]=useState(position?.track??0),[finished,setFinished]=useState(position?.finished??false);
 const [playing,setPlaying]=useState(false),[volume,setVolume]=useState(initialVolume),[warning,setWarning]=useState("");
 const continuePlay=useRef(false),started=useRef(position?.started??false),lastSave=useRef(-1);
 const latest=useRef(onPosition);latest.current=onPosition;
 const ended=useRef(false);
 const start=(ranges[track]?.startMs??0)/1000,end=ranges[track]?.endMs===undefined?undefined:ranges[track]!.endMs!/1000;
 const persist=(seconds=audio.current?.currentTime??0,t=track,end=finished)=>latest.current?.({track:t,seconds,finished:end,started:started.current});
 const advance=()=>{if(ended.current)return;ended.current=true;audio.current?.pause();setPlaying(false);if(track+1<urls.length){continuePlay.current=true;persist((ranges[track+1]?.startMs??0)/1000,track+1,false);lastSave.current=-1;setTrack(track+1);}else{setFinished(true);persist(end??audio.current?.duration??0,track,true);}};
 useEffect(()=>{ended.current=false;},[track]);
 useEffect(()=>setVolume(initialVolume),[initialVolume]);
 useEffect(()=>{if(audio.current)audio.current.volume=volume;},[volume,urls[track]]);
 useEffect(()=>{const a=audio.current;if(!a)return;setWarning("");if("setSinkId" in a)(a as HTMLAudioElement&{setSinkId:(id:string)=>Promise<void>}).setSinkId(outputDeviceId).catch(()=>setWarning("所选播放设备不可用，请回设置重新选择。"));},[outputDeviceId,urls[track]]);
 useEffect(()=>{if(paused){audio.current?.pause();setPlaying(false);continuePlay.current=false;}},[paused]);
 useEffect(()=>{
   if(!continuePlay.current||paused||finished||!urls[track])return;
   continuePlay.current=false;
   audio.current?.play().then(()=>setPlaying(true)).catch(()=>{setPlaying(false);onError("下一段自动播放失败；已保留当前位置，请重试播放。");});
 },[track,urls[track],paused,finished]);
 if(!urls.length)return <p role="alert">听力音频正在准备；准备好后可以播放。</p>;
 return <div className="audio-player">
  <audio ref={audio} src={urls[track]} onCanPlay={onReady}
    onLoadedMetadata={()=>{const p=position;const a=audio.current;if(a)a.currentTime=p?.track===track&&p.seconds>=start&&(end===undefined||p.seconds<end)?p.seconds:start;}}
    onTimeUpdate={()=>{if(end!==undefined&&(audio.current?.currentTime??0)>=end){advance();return;}const n=Math.floor(audio.current?.currentTime??0);if(n!==lastSave.current){lastSave.current=n;persist();}}}
    onError={()=>onError("音频无法解码或播放，请检查卷包。")}
    onEnded={advance}/>
  <button disabled={paused||(mode==="exam"&&(playing||finished))} onClick={async()=>{
   try{if(playing&&mode==="training"){audio.current?.pause();setPlaying(false);persist();}
   else{if(finished&&mode==="training"){setFinished(false);ended.current=false;if(audio.current)audio.current.currentTime=start;}
    await audio.current?.play();started.current=true;setPlaying(true);persist();}}
   catch{onError("播放失败，当前位置已保留，请检查输出设备后重试。");}
  }}>{finished&&mode==="exam"?"听力播放已完成":playing?"播放中":`播放音频 ${track+1}/${urls.length}`}</button>
  {warning&&<span role="status">{warning}</span>}
  <label>音量 <input aria-label="音量" type="range" min="0" max="1" step="0.05" value={volume} onChange={e=>setVolume(Number(e.target.value))}/></label>
  {mode==="training"&&<button onClick={()=>{if(audio.current)audio.current.currentTime=start;ended.current=false;setFinished(false);persist(start,track,false);}}>重听本段</button>}
 </div>;
}
