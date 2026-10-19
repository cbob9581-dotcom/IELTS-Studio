import {test,expect,vi,afterEach} from "vitest";
import {render,waitFor,fireEvent,screen} from "@testing-library/react";
import {useState} from "react";
import {SpeakingSession} from "./SpeakingSession";
import {completeFixture} from "../../test/fixtures/complete-paper";
import {createAttempt} from "../domain/session";
import {createSpeakingState} from "./flow";
import {defaultConfig} from "../configuration/types";
import type {DesktopPort} from "../platform/desktop";
const mocks=vi.hoisted(()=>({begin:vi.fn(),suspend:vi.fn(),dispose:vi.fn().mockResolvedValue(undefined)}));
vi.mock("./audio",async original=>({...await original<object>(),Microphone:{open:async()=>mocks}}));
vi.mock("../rooms/SpeakingRoom",()=>({SpeakingRoom:({onReady}:{onReady:()=>void})=><button onClick={onReady}>场景就绪</button>}));
afterEach(()=>{vi.clearAllMocks();});
function setup(finished=false){
 const paper=completeFixture(),a=createAttempt(paper,"training","speaking");
 a.status="answering";a.speaking={...createSpeakingState(paper.sections[3].questions,{part:1},0),stage:finished?"finished":"answering",deadline:Date.now()+90000};
 const config=defaultConfig();for(const id of ["deepseek","qwenTts","qwenAsr"] as const)Object.assign(config.providers[id],{enabled:true,baseUrl:"https://example.com",model:"test",voice:"Serena"});
 const save=vi.fn().mockResolvedValue(undefined),recognize=vi.fn().mockResolvedValue("answer"),complete=vi.fn().mockRejectedValueOnce(Error("disk unavailable")).mockResolvedValue(undefined);
 const port={credentialStatus:async()=>({deepseek:true,qwenAsr:true,qwenTts:true}),saveSpeakingRecording:save,recognizeSpeech:recognize,cancelSpeech:async()=>{}} as unknown as DesktopPort;
 let exit:()=>Promise<void>=async()=>{},latest=a;
 function Harness(){const [attempt,set]=useState(a);latest=attempt;return <SpeakingSession paper={paper} attempt={attempt} port={port} config={config} interactive onChange={set} onComplete={complete} registerBeforeExit={fn=>{exit=fn;return()=>{};}}/>;}
 render(<Harness/>);fireEvent.click(screen.getByText("场景就绪"));
 return{save,complete,recognize,exit:()=>exit(),latest:()=>latest};
}
test("exit waits for current full recording to be saved and persists a recoverable take",async()=>{
 const wav=new ArrayBuffer(524);new DataView(wav).setUint32(24,24000,true);
 mocks.suspend.mockReturnValueOnce({wav,voiced:true}).mockReturnValue(null);
 const s=setup();await waitFor(()=>expect(mocks.begin).toHaveBeenCalled());
 await s.exit();
 await waitFor(()=>expect(s.latest().speakingCapture?.finalId).toBeTruthy());
 expect(s.save).toHaveBeenCalledTimes(1);expect(s.recognize).not.toHaveBeenCalled();
});
test("failed final submission can be retried without leaving speaking room",async()=>{
 mocks.suspend.mockReturnValue(null);
 const s=setup(true);await screen.findByText("disk unavailable");
 fireEvent.click(screen.getByText("重试当前环节"));
 await waitFor(()=>expect(s.complete).toHaveBeenCalledTimes(2));
});
