import type { Question } from "../domain/paper";
import type { Scope } from "../domain/session";
export type SpeakingStage = "greeting" | "question" | "answering" | "instructions" | "preparation" | "begin_long_turn" | "long_turn" | "roundoff_question" | "roundoff_answer" | "closing" | "finished";
export type SpeakingTurn = { turnId:string; questionId:string; prompt:string; text:string; recordingId:string; at:number };
export type SpeakingState = {
  version:1; stage:SpeakingStage; sequence:number; turnId:string;
  questions:Question[]; index:number; handoffReady:boolean; instructionEnded:boolean;
  deadline:number|null; partStartedAt:number|null; startedAt:number; turns:SpeakingTurn[];
  partElapsedMs?:number; answerStartedAt?:number;
  utterance?:string;
};
export type SpeakingEvent =
  | {type:"audio_ended";turnId:string;now:number;durationMs?:number}
  | {type:"props_ready";now:number}
  | {type:"tick";now:number}
  | {type:"utterance";turnId:string;text:string;now:number}
  | {type:"answer";turnId:string;text:string;recordingId:string;now:number;durationMs?:number};
function enter(s:SpeakingState,stage:SpeakingStage,now:number,index=s.index):SpeakingState {
  const sequence=s.sequence+1;
  return {...s,stage,index,sequence,turnId:`speaking-${sequence}`,utterance:stage==="answering"||stage==="roundoff_answer"?s.utterance:undefined,
    answerStartedAt:["answering","long_turn","roundoff_answer"].includes(stage)?now:undefined,
    deadline:stage==="preparation"?now+60000:stage==="long_turn"?now+120000:["answering","roundoff_answer"].includes(stage)?now+90000:null};
}
function question(s:SpeakingState,now:number,index:number):SpeakingState {
  if(index>=s.questions.length) return enter(s,"closing",now);
  const part=s.questions[index].part;
  const next={...s,partElapsedMs:s.questions[s.index]?.part===part?(s.partElapsedMs??0):0,partStartedAt:s.questions[s.index]?.part===part?s.partStartedAt:now};
  return enter(next,part===2?"instructions":"question",now,index);
}
export function createSpeakingState(questions:Question[],scope:Scope,now:number):SpeakingState {
  const selected=questions.filter(q=>(!scope.part||q.part===scope.part)&&(!scope.kind||q.kind===scope.kind));
  if(!selected.length)throw Error("此范围没有口语题目");
  const initial:SpeakingState={version:1,stage:"greeting",sequence:0,turnId:"speaking-0",questions:selected,index:0,handoffReady:false,instructionEnded:false,deadline:null,partElapsedMs:0,partStartedAt:now,startedAt:now,turns:[]};
  return scope.part?question(initial,now,0):initial;
}
export function reduceSpeaking(s:SpeakingState,e:SpeakingEvent):SpeakingState {
  if(s.stage==="finished")return s;
  if(e.type==="utterance")return e.turnId===s.turnId&&["question","roundoff_question"].includes(s.stage)?{...s,utterance:e.text}:s;
  if(e.type==="props_ready") {
    if(s.handoffReady)return s;
    const next={...s,handoffReady:true};
    return next.stage==="instructions"&&next.instructionEnded?enter(next,"preparation",e.now):next;
  }
  if(e.type==="audio_ended") {
    if(e.turnId!==s.turnId)return s;
    s={...s,partElapsedMs:(s.partElapsedMs??0)+Math.max(0,e.durationMs??0)};
    if(s.stage==="greeting")return question(s,e.now,0);
    if(s.stage==="question")return enter(s,"answering",e.now);
    if(s.stage==="roundoff_question")return enter(s,"roundoff_answer",e.now);
    if(s.stage==="instructions") {
      const next={...s,instructionEnded:true};
      return next.handoffReady?enter(next,"preparation",e.now):next;
    }
    if(s.stage==="begin_long_turn")return enter(s,"long_turn",e.now);
    if(s.stage==="closing")return enter(s,"finished",e.now);
    return s;
  }
  if(e.type==="tick") {
    if(s.stage==="preparation"&&s.deadline!==null&&e.now>=s.deadline) return enter(s,"begin_long_turn",e.now);
    // Recording is stopped at this deadline by the audio coordinator; its saved answer advances the flow.
    return s;
  }
  if(e.type==="answer"&&e.turnId===s.turnId&&["answering","long_turn","roundoff_answer"].includes(s.stage)) {
    const q=s.questions[s.index];
    const elapsed=(s.partElapsedMs??0)+Math.max(0,e.durationMs??e.now-(s.answerStartedAt??e.now));
    const next={...s,partElapsedMs:elapsed,turns:[...s.turns,{turnId:s.turnId,questionId:q.id,prompt:s.utterance??q.prompt,text:e.text,recordingId:e.recordingId,at:e.now}]};
    if(s.stage==="long_turn")return enter(next,"roundoff_question",e.now);
    let index=s.index+1;
    const partTurns=next.turns.filter(t=>s.questions.find(x=>x.id===t.questionId)?.part===q.part).length;
    if(q.part!==2&&s.partStartedAt!==null&&elapsed<240000&&partTurns<12&&s.questions[index]?.part!==q.part)
      index=s.index; // Cloud examiner generates a follow-up on this part's frozen topic.
    // Parts 1 and 3 are bounded by their real stage clock; only skip the same part, never another requested part.
    if(q.part!==2&&s.partStartedAt!==null&&elapsed>=300000)
      while(index<s.questions.length&&s.questions[index].part===q.part)index++;
    return question(next,e.now,index);
  }
  return s;
}
export function examinerText(s:SpeakingState):string|null {
  if(s.stage==="greeting")return "Good morning. Welcome to your speaking test. Let's begin.";
  if(s.stage==="instructions")return "Now I'm going to give you a topic. I'd like you to talk about it for one to two minutes. You have one minute to think about what you're going to say. You can make notes on this paper if you wish. Here is your topic.";
  if(s.stage==="begin_long_turn")return "All right. Remember, you have one to two minutes for this, so don't worry if I stop you. Please begin.";
  if(s.stage==="question")return s.questions[s.index].prompt;
  if(s.stage==="roundoff_question")return s.questions[s.index].prompt;
  if(s.stage==="closing")return "Thank you. That is the end of the speaking test.";
  return null;
}
