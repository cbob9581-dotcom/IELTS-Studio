import {createRoot} from "react-dom/client";
import {useState} from "react";
import {ListeningPlayer,type ListeningPosition} from "../../src/exam/ListeningPlayer";
import {encodeWave} from "../../src/speaking/audio";
if(import.meta.env.MODE!=="testbrowser")throw Error("Test-only fixture");
const urls=[300,500].map(f=>URL.createObjectURL(new Blob([encodeWave([Float32Array.from({length:6000},(_,i)=>Math.sin(i/24000*f*Math.PI*2)*.03)],24000)],{type:"audio/wav"})));
function Fixture(){const [position,set]=useState<ListeningPosition|undefined>(()=>JSON.parse(localStorage.getItem("listening-position")??"null")??undefined);
return <ListeningPlayer urls={urls} mode="exam" paused={false} position={position} onPosition={p=>{set(p);localStorage.setItem("listening-position",JSON.stringify(p));}} onReady={()=>{}} onError={e=>{throw Error(e);}}/>;}
createRoot(document.getElementById("root")!).render(<Fixture/>);
