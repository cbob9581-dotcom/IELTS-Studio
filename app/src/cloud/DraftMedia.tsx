import {useState,useEffect} from "react";
import type {CloudJob} from "./types";
import type {DesktopPort} from "../platform/desktop";
import {errorText} from "../platform/desktop";
export function DraftMedia({job,port}:{job:CloudJob;port:DesktopPort}){
 const [urls,setUrls]=useState<Record<string,string>>({}),[error,setError]=useState(""),[opened,setOpened]=useState(false);
 const [assetId,setAssetId]=useState("");
 useEffect(()=>{let active=true;const created:string[]=[];const assets=job.assets??[];
   setUrls({});setError("");
   if(opened&&assetId)Promise.all(assets.filter(a=>a.id===assetId).map(async a=>{const bytes=await port.cloudAsset(job.id,a.id);if(!active)return [a.id,""];const url=URL.createObjectURL(new Blob([bytes],{type:a.mediaType}));created.push(url);return [a.id,url];})).then(entries=>{if(active)setUrls(Object.fromEntries(entries));}).catch(e=>{if(active)setError(errorText(e));});
   return()=>{active=false;created.forEach(u=>URL.revokeObjectURL(u));};
 },[job.id,job.assets,port,opened,assetId]);
 return <details onToggle={e=>setOpened(e.currentTarget.open)}><summary>预览已生成 / 上传的图片和音频（核对用途）</summary><p>选择一项后加载，避免同时读取整卷录音。</p>{error&&<p className="error">{error}</p>}<div className="cloud-media">{job.assets?.map(a=><figure key={a.id}><figcaption>{a.id}</figcaption><button onClick={()=>setAssetId(a.id)}>加载{a.mediaType.startsWith("audio/")?"录音":"图片"} {a.id}</button>{urls[a.id]&&(a.mediaType.startsWith("audio/")?<audio controls preload="none" src={urls[a.id]}/>:<img src={urls[a.id]} alt={a.id}/>)}</figure>)}</div></details>;
}
