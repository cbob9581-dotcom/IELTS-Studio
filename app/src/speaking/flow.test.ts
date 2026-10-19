import { expect, test } from "vitest";
import { createSpeakingState, reduceSpeaking } from "./flow";
import type { Question } from "../domain/paper";
const questions = [1,2,3].map(part => ({id:`s${part}`,number:part,part,kind:"speaking_prompt",prompt:`Question ${part}`,points:0,evidence:[]})) as Question[];

test("Part 2 only starts one preparation clock after both real audio and props", () => {
  let s = createSpeakingState(questions, {part:2}, 1000);
  expect(s.stage).toBe("instructions");
  expect(s.deadline).toBeNull();
  s = reduceSpeaking(s,{type:"props_ready",now:2000});
  expect(s.stage).toBe("instructions");
  s = reduceSpeaking(s,{type:"audio_ended",turnId:s.turnId,now:5000});
  expect(s.stage).toBe("preparation"); expect(s.deadline).toBe(65000);
  s = reduceSpeaking(s,{type:"props_ready",now:6000});
  expect(s.deadline).toBe(65000);
  s = reduceSpeaking(s,{type:"tick",now:65000});
  expect(s.stage).toBe("begin_long_turn");
  expect(s.deadline).toBeNull();
  s = reduceSpeaking(s,{type:"audio_ended",turnId:s.turnId,now:67000});
  expect(s.stage).toBe("long_turn");expect(s.deadline).toBe(187000);
});
test("stale audio completion cannot advance a new examiner turn", () => {
  let s = createSpeakingState(questions,{},0); const id=s.turnId;
  s=reduceSpeaking(s,{type:"audio_ended",turnId:id,now:1000});
  expect(s.stage).toBe("question");
  const same=reduceSpeaking(s,{type:"audio_ended",turnId:id,now:2000});
  expect(same.stage).toBe("question");expect(same.turnId).toBe(s.turnId);
});
test("Part 2 training ends without accidentally starting Part 3",()=>{
  let s=createSpeakingState(questions,{part:2},0);
  s=reduceSpeaking(s,{type:"props_ready",now:0});
  s=reduceSpeaking(s,{type:"audio_ended",turnId:s.turnId,now:1000});
  s=reduceSpeaking(s,{type:"tick",now:61000});
  s=reduceSpeaking(s,{type:"audio_ended",turnId:s.turnId,now:62000});
  s=reduceSpeaking(s,{type:"answer",turnId:s.turnId,text:"My answer",recordingId:"take-1",now:170000});
  expect(s.stage).toBe("roundoff_question");
  s=reduceSpeaking(s,{type:"audio_ended",turnId:s.turnId,now:170500});
  expect(s.stage).toBe("roundoff_answer");
  s=reduceSpeaking(s,{type:"answer",turnId:s.turnId,text:"One more point",recordingId:"take-2",now:170800});
  expect(s.stage).toBe("closing");
  s=reduceSpeaking(s,{type:"audio_ended",turnId:s.turnId,now:171000});
  expect(s.stage).toBe("finished");expect(s.turns[0].recordingId).toBe("take-1");
});
test("repeated Part 1 answers cannot extend the same part beyond its time boundary",()=>{
 let s=createSpeakingState(questions,{part:1},0);
 s=reduceSpeaking(s,{type:"audio_ended",turnId:s.turnId,now:1000});
 s=reduceSpeaking(s,{type:"answer",turnId:s.turnId,text:"Answer",recordingId:"r1",now:2000});
 expect(s.stage).toBe("question");
 s=reduceSpeaking(s,{type:"audio_ended",turnId:s.turnId,now:300000});
 s=reduceSpeaking(s,{type:"answer",turnId:s.turnId,text:"Last answer",recordingId:"r2",now:301000,durationMs:300000});
 expect(s.stage).toBe("closing");
});
test("JSON recovery preserves handoff and expired preparation goes to prompt once",()=>{
  let s=createSpeakingState(questions,{part:2},0);
  s=reduceSpeaking(s,{type:"audio_ended",turnId:s.turnId,now:1000});
  expect(s.stage).toBe("instructions");
  s=reduceSpeaking(s,{type:"props_ready",now:2000});
  s=JSON.parse(JSON.stringify(s));
  s=reduceSpeaking(s,{type:"tick",now:100000});
  const id=s.turnId;
  s=reduceSpeaking(s,{type:"tick",now:101000});
  expect(s.turnId).toBe(id);expect(s.handoffReady).toBe(true);
});
