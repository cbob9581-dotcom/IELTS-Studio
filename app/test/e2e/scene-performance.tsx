// Test-only real GLB rendering; no provider requests or user records.
import React,{useEffect,useRef,useState} from 'react';
import {createRoot} from 'react-dom/client';
import {SpeakingRoom} from '../../src/rooms/SpeakingRoom';
import {emptyNotes} from '../../src/rooms/paperSurfaces';
import {createSpeakingState,type SpeakingStage} from '../../src/speaking/flow';
import {browserTestPort} from '../../src/platform/browser-test';
import speaking from '../fixtures/dev-speaking.json';
import type {Question} from '../../src/domain/paper';
import roomUrl from '../../../rooms/speaking/runtime/speaking-room.glb?url';
import '../../src/styles.css';
let reads=0;
const port={...browserTestPort,readRuntimeModel:async()=>{reads++;return (await fetch(roomUrl)).arrayBuffer();}};
function Fixture(){
 const [mounted,setMounted]=useState(true),[ready,setReady]=useState(false),[notes,setNotes]=useState(emptyNotes);
 const [flow,setFlow]=useState(()=>createSpeakingState(speaking.sections[0].questions as Question[],{part:2},Date.now()));const mouth=useRef(0);
 useEffect(()=>{let index=0;const phases:SpeakingStage[]=['question','answering','instructions','preparation','begin_long_turn','long_turn','closing'];const timer=setInterval(()=>setFlow(f=>({...f,stage:phases[index++%phases.length],sequence:f.sequence+1,turnId:`benchmark-${index}`,handoffReady:false,deadline:null})),12000);return()=>clearInterval(timer);},[]);
 useEffect(()=>{let frame=0;const animate=()=>{mouth.current=['question','instructions','begin_long_turn','closing','greeting'].includes(flow.stage)?Math.max(0,Math.sin(performance.now()/110))*.6:0;frame=requestAnimationFrame(animate);};frame=requestAnimationFrame(animate);return()=>cancelAnimationFrame(frame);},[flow.stage]);
 return <div className="game-session"><div className="game-world">{mounted&&<SpeakingRoom port={port} flow={flow} topic="Describe a quiet place you enjoy visiting." notes={notes} onNotes={setNotes} onPropsReady={()=>setFlow(f=>({...f,handoffReady:true}))} onReady={()=>setReady(true)} mouthLevel={mouth}/>}</div><div style={{position:'fixed',top:4,right:4,zIndex:30,background:'white'}}><button onClick={()=>{setReady(false);setMounted(m=>!m);}}>Toggle room</button><output id="benchmark-ready">{mounted?ready?'ready':'loading':'unmounted'}</output><output id="benchmark-loads">{reads}</output><output id="benchmark-phase">{flow.stage}</output></div></div>;
}
if(import.meta.env.MODE!=='testbrowser')throw Error('Performance fixture is unavailable in production');
createRoot(document.getElementById('root')!).render(<Fixture/>);
