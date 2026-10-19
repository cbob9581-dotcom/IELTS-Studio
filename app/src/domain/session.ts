import { checkReadiness } from "./readiness";
import type { SpeakingState } from "../speaking/flow";
import type {
  PaperVersion,
  Section,
  Mode,
  SectionPaper,
  QuestionKind,
  Response,
} from "./paper";
export type AttemptClock = {
  durationMs: number;
  deadlineUtcMs: number | null;
  lastObservedUtcMs: number;
  remainingMs: number;
};
export type Scope = { part?: number; kind?: QuestionKind };
export type Annotation = {
  id: string;
  paragraphId: string;
  start: number;
  end: number;
  text: string;
  note: string;
};
export type AttemptSnapshot = {
  id: string;
  paperId: string;
  paperVersion: number;
  examRunId: string | null;
  mode: Mode;
  section: Section;
  status: "room" | "answering" | "interrupted" | "submitted";
  responses: Response[];
  annotations: Annotation[];
  revision: number;
  clock: AttemptClock;
  interruptionEvents: { at: number; reason: string }[];
  scope: Scope;
  flagged: string[];
  updatedAt?: number;
  submittedAt?: number;
  revealedQuestions?: string[];
  environmentEvents?:{at:number;kind:string}[];
  listening?: { track: number; seconds: number; finished: boolean; started: boolean };
  speakingCapture?: { turnId:string; takeId:string; chunks:string[]; finalId?:string; voiced?:boolean; durationMs?:number };
  speaking?: SpeakingState;
  speakingNotes?: { text:string; strokes:{x:number;y:number;start?:boolean}[] };
};
export type SessionState = {
  phase:
    | "selecting"
    | "room"
    | "answering"
    | "submitting"
    | "submitted"
    | "interrupted";
  mode: Mode;
  section: Section;
  clock: AttemptClock;
};
export type SessionEvent =
  | { type: "OPEN"; now: number }
  | { type: "SUBMIT" }
  | { type: "SUBMITTED" }
  | { type: "INTERRUPT" }
  | { type: "RESUME" };
export function beginSession(
  paper: PaperVersion,
  mode: Mode,
  section: Section = "listening",
): SessionState {
  const issues = checkReadiness(paper, mode, section);
  if (issues.length) throw new Error(issues.map((x) => x.message).join("；"));
  const duration = paper.sections.find(
    (s) => s.section === section,
  )!.durationMs;
  return {
    phase: "room",
    mode,
    section,
    clock: {
      durationMs: duration,
      deadlineUtcMs: null,
      lastObservedUtcMs: Date.now(),
      remainingMs: duration,
    },
  };
}
export function reduceSession(s: SessionState, e: SessionEvent): SessionState {
  if (e.type === "OPEN" && s.phase === "room")
    return {
      ...s,
      phase: "answering",
      clock: {
        ...s.clock,
        deadlineUtcMs: s.mode === "exam" ? e.now + s.clock.remainingMs : null,
        lastObservedUtcMs: e.now,
      },
    };
  if (e.type === "SUBMIT" && ["answering", "interrupted"].includes(s.phase))
    return { ...s, phase: "submitting" };
  if (e.type === "SUBMITTED" && s.phase === "submitting")
    return { ...s, phase: "submitted" };
  if (e.type === "INTERRUPT" && s.phase === "answering")
    return { ...s, phase: "interrupted" };
  if (e.type === "RESUME" && s.phase === "interrupted")
    return { ...s, phase: "answering" };
  return s;
}
export function restoreClock(c: AttemptClock, now: number): AttemptClock {
  return {
    ...c,
    remainingMs:
      c.deadlineUtcMs === null
        ? c.remainingMs
        : Math.max(0, Math.min(c.remainingMs, c.deadlineUtcMs - now)),
    lastObservedUtcMs: Math.max(c.lastObservedUtcMs, now),
  };
}
export function advanceClock(
  c: AttemptClock,
  now: number,
  elapsedMs: number,
): AttemptClock {
  const restored = restoreClock(c, now);
  return {
    ...restored,
    remainingMs: Math.max(
      0,
      Math.floor(
        Math.min(restored.remainingMs, c.remainingMs - Math.max(0, elapsedMs)),
      ),
    ),
  };
}
export function nextSection(s: Section): Section | null {
  const order: Section[] = ["listening", "reading", "writing", "speaking"];
  return order[order.indexOf(s) + 1] ?? null;
}
export function createNextExamAttempt(p:PaperVersion,previous:AttemptSnapshot):AttemptSnapshot|null{
  if(previous.mode!=="exam"||previous.status!=="submitted"||!previous.examRunId||previous.paperId!==p.id||previous.paperVersion!==p.version)throw Error("仅已提交且绑定同一套卷的考试可进入下一项");
  const next=nextSection(previous.section);return next?createAttempt(p,"exam",next,{},previous.examRunId):null;
}
export function selectScope(s: SectionPaper, scope: Scope): SectionPaper {
  let audioAssetIds=s.audioAssetIds;
  let audioPlaylist=s.audioPlaylist;
  if(scope.part&&s.section==='listening'&&audioPlaylist?.length){
    audioPlaylist=audioPlaylist.flatMap(track=>{
      if(track.part===scope.part)return [track];
      if(track.part!==null)return [];
      const range=track.segments.find(s=>s.part===scope.part);
      if(!range)throw Error('此整套听力音频尚未标记所选 Part 的起止时间，请在听力音频中标记后练习，或选择完整听力。');
      return [{...track,startMs:range.startMs,endMs:range.endMs}];
    });
    if(!audioPlaylist.length)throw Error('所选 Part 没有绑定音频。');
    audioAssetIds=audioPlaylist.map(t=>t.assetId);
  }else if(scope.part&&s.section==="listening"&&audioAssetIds?.length){
    const parts=new Set(s.questions.map(q=>q.part));
    const mapping=s.audioParts??Object.fromEntries(audioAssetIds.flatMap(id=>{
      const match=/^listening-([1-4])-\d+$/.exec(id);return match?[[id,Number(match[1])]]:[];
    }));
    if(parts.size>1&&audioAssetIds.some(id=>!mapping[id]))throw Error("此旧卷尚未标记音频所属 Part，请在题库核对音频 Part 对应关系后练习，或先选择整项听力。");
    audioAssetIds=audioAssetIds.filter(id=>mapping[id]===scope.part||parts.size===1);
    if(!audioAssetIds.length)throw Error("所选 Part 没有绑定音频，请先在题库补充对应录音。");
  }
  let durationMs=s.durationMs;
  if(scope.part&&s.section==='listening'&&audioPlaylist?.length){const durations=audioPlaylist.map(t=>(t.endMs??t.durationMs??NaN)-(t.startMs??0));if(durations.every(n=>Number.isFinite(n)&&n>0))durationMs=Math.max(1000,Math.round(durations.reduce((a,b)=>a+b,0)));}
  return {
    ...s,audioAssetIds,audioPlaylist,durationMs,
    questions: s.questions.filter(
      (q) =>
        (!scope.part || q.part === scope.part) &&
        (!scope.kind || q.kind === scope.kind),
    ),
    paragraphs: s.paragraphs.filter(
      (p) => !scope.part || p.part === scope.part,
    ),
  };
}
export function createAttempt(
  p: PaperVersion,
  mode: Mode,
  section: Section,
  scope: Scope = {},
  examRunId: string | null = null,
): AttemptSnapshot {
  const s = beginSession(p, mode, section);
  const selected=selectScope(
      p.sections.find((x) => x.section === section)!,
      scope,
    );
  if (!selected.questions.length)
    throw new Error("此练习范围没有题目");
  if(mode==='training'&&section==='listening'&&scope.part&&selected.audioPlaylist?.length){s.clock={...s.clock,durationMs:selected.durationMs,remainingMs:selected.durationMs};}
  return {
    id: crypto.randomUUID(),
    paperId: p.id,
    paperVersion: p.version,
    examRunId: mode === "exam" ? (examRunId ?? crypto.randomUUID()) : null,
    mode,
    section,
    status: "room",
    responses: [],
    annotations: [],
    revision: 0,
    clock: s.clock,
    interruptionEvents: [],
    scope,
    flagged: [],
  };
}
