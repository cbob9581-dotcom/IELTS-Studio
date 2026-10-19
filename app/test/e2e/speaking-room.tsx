// Test-only entry. Synthetic external service results are never imported by the app entry.
import React,{useState} from "react";
import {createRoot} from "react-dom/client";
import { SpeakingSession } from "../../src/speaking/SpeakingSession";
import { browserTestPort } from "../../src/platform/browser-test";
import { defaultConfig } from "../../src/configuration/types";
import {createAttempt,type AttemptSnapshot} from "../../src/domain/session";
import {reduceSpeaking} from "../../src/speaking/flow";
import {encodeWave} from "../../src/speaking/audio";
import speaking from "../fixtures/dev-speaking.json";
import type {PaperVersion} from "../../src/domain/paper";
import roomUrl from "../../../rooms/speaking/runtime/speaking-room.glb?url";
import "../../src/styles.css";
const paper=speaking as PaperVersion;
const config=defaultConfig();
for(const id of ["deepseek","qwenAsr","qwenTts"] as const){config.providers[id].enabled=true;config.providers[id].model="TEST FIXTURE";config.providers[id].baseUrl="https://test.invalid";}
config.providers.qwenTts.voice="TEST FIXTURE";
if(new URLSearchParams(location.search).has("missing-service"))config.providers.deepseek.enabled=false;
const testTracks:MediaStreamTrack[]=[];
const openDevice=navigator.mediaDevices.getUserMedia.bind(navigator.mediaDevices);
navigator.mediaDevices.getUserMedia=async constraints=>{const stream=await openDevice(constraints);testTracks.push(...stream.getTracks());return stream;};
const tone=encodeWave([new Float32Array(24000).map((_,i)=>Math.sin(i*.05)*.01)],24000);
const port={...browserTestPort,credentialStatus:async()=>({deepseek:true,qwenAsr:true,qwenTts:true}),
 readRuntimeModel:async()=>{
   const buffer=await (await fetch(roomUrl)).arrayBuffer();
   const params=new URLSearchParams(location.search),drop=params.get("hide"),noNormal=params.has("no-normal");if(!drop&&!noNormal)return buffer;
   const view=new DataView(buffer),n=view.getUint32(12,true);
   const doc=JSON.parse(new TextDecoder().decode(new Uint8Array(buffer,20,n)));
   for(const material of doc.materials){if(drop&&material.name.toLowerCase().includes(drop)){
     material.pbrMetallicRoughness.baseColorFactor=[1,1,1,0];material.alphaMode="BLEND";
   }if(noNormal&&material.name.includes("RebuiltHair"))delete material.normalTexture;}
   const encoded=new TextEncoder().encode(JSON.stringify(doc)),size=Math.ceil(encoded.length/4)*4;
   const result=new Uint8Array(20+size+buffer.byteLength-20-n);result.set(new Uint8Array(buffer,0,20));
   new DataView(result.buffer).setUint32(8,result.length,true);new DataView(result.buffer).setUint32(12,size,true);
   result.fill(32,20,20+size);result.set(encoded,20);result.set(new Uint8Array(buffer,20+n),20+size);return result.buffer;
 },synthesizeSpeech:async()=>tone,
 recognizeSpeech:async()=>"TEST FIXTURE answer",examinerQuestion:async()=>"TEST FIXTURE question",
 saveSpeakingRecording:async()=>{},cancelSpeech:async()=>{}};
function Fixture(){
 const [attempt,setAttempt]=useState<AttemptSnapshot>(()=>{
   const a:AttemptSnapshot=JSON.parse(localStorage.getItem("speaking-fixture")??"null")??createAttempt(paper,"training","speaking",{part:2});
   if(new URLSearchParams(location.search).has("finish"))a.speaking={version:1,stage:"finished",sequence:8,turnId:"speaking-8",questions:paper.sections[0].questions,index:0,handoffReady:true,instructionEnded:true,deadline:null,partStartedAt:Date.now(),startedAt:Date.now(),turns:[]};
   return a;
 });
 const change=(a:AttemptSnapshot)=>{setAttempt(a);localStorage.setItem("speaking-fixture",JSON.stringify(a));};
 return <div className="game-session"><div className="game-world">
 <SpeakingSession port={port} paper={paper} attempt={attempt} config={config} onChange={change} onComplete={async()=>{change({...attempt,status:"submitted"});}} interactive/>
 </div><output className="test-stage" style={{position:"absolute",right:12,top:12,background:"#000",color:"white"}}>{attempt.speaking?.stage??"loading"}</output>
 <button hidden id="track-state" onClick={e=>{e.currentTarget.dataset.states=testTracks.map(t=>t.readyState).join(",");}}/>
 </div>;
}
if(import.meta.env.MODE!=="testbrowser")throw Error("Test fixture is unavailable in production");
createRoot(document.getElementById("root")!).render(<Fixture/>);
