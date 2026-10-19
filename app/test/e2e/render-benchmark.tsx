// Isolated renderer benchmark: real asset, animated jaw, blink, camera and handoff; no cloud requests.
import React,{useRef} from "react";
import {createRoot} from "react-dom/client";
import {SpeakingRoom} from "../../src/rooms/SpeakingRoom";
import {browserTestPort} from "../../src/platform/browser-test";
import {emptyNotes} from "../../src/rooms/paperSurfaces";
import roomUrl from "../../../rooms/speaking/runtime/speaking-room.glb?url";
import "../../src/styles.css";
if(import.meta.env.MODE!=="testbrowser")throw Error("Test only");
const port={...browserTestPort,readRuntimeModel:async()=>(await fetch(roomUrl)).arrayBuffer()};
const notes=emptyNotes();
const stage=new URLSearchParams(location.search).get("stage")??"instructions";
function Benchmark(){const mouth=useRef(0);return <div className="game-session"><div className="game-world"><SpeakingRoom port={port} flow={{stage} as never} topic="Benchmark card" notes={notes} onNotes={()=>{}} onPropsReady={()=>{}} onReady={()=>{document.body.dataset.ready="true";const tick=()=>{mouth.current=stage==="answering"?0:.5+.45*Math.sin(performance.now()*.015);requestAnimationFrame(tick);};tick();}} mouthLevel={mouth}/></div></div>;}
createRoot(document.getElementById("root")!).render(<Benchmark/>);
