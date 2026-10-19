import type {
  Section,
  Question,
  SectionPaper,
  AssetRef,
  SourceAnchor,
  AnswerRule,
} from "../domain/paper";
export type SourceRole =
  | "questions"
  | "answers"
  | "explanations"
  | "essays"
  | "transcript"
  | "audio"
  | "mixed";
export type Block = {
  id: string;
  page: number | null;
  type: string;
  text: string;
  imagePath?: string;
  imageAssetId?: string;
  bbox?: number[] | null;
  auxiliaryImage?:boolean;
  auxiliaryConfirmed?:boolean;
};
export type SourceDocument = {
  id: string;
  name: string;
  extension: string;
  roles: SourceRole[];
  state: string;
  blockCount: number;
  pageCount: number;
  scanCursor: number;
  unassignedRanges?: {start:number;end:number;reason:string}[];
  error?: string;
  assets?: AssetRef[];
  binding?: AudioBinding;
  durationMs?:number;
  audioVerified?:boolean;
  origin?:'book'|'user'|'uploaded';
  operation?: { label: string; done?: number; total?: number };
  progress?: unknown;
};
export type SourceQuestion = Question & {
  conflicts?: Entry[];
  answerSource?: Entry;
};
export type Unit = {
  id: string;
  documentId: string;
  title: string;
  test: string;
  section: Section;
  part: number;
  role: SourceRole;
  start: number;
  end: number;
  state: string;
  confirmed: boolean;
  error?: string;
  result?: {
    section?: Omit<SectionPaper, "questions"> & { questions: SourceQuestion[] };
    warnings?: string[];
    entries?: number;
  };
};
export type Entry = {
  id: string;
  unitId: string;
  kind: SourceRole;
  test: string;
  section: Section;
  numbers: number[];
  text: string;
  quote: string;
  answer?: AnswerRule;
  anchor: SourceAnchor;
  state: string;
  candidates?: string[];
};
export type SourceProject = {
  id: string;
  title: string;
  revision: number;
  documents: SourceDocument[];
  units: Unit[];
  entries: Entry[];
  audioDrafts?:ScriptDraft[];
  publications: {
    test: string;
    paperId: string;
    version: number;
    at: number;
  }[];
};
export type AudioBinding={test:string;part?:number;scope?:'test'|'part';active?:boolean;order?:number;segments?:{part:number;startMs:number;endMs:number}[]};
export type ScriptDraft={id:string;revision:number;origin:'book'|'user';test:string;part:number;parallelism?:number;rawText:string;turns:{id:string;speaker:string;text:string}[];events:({type:'speech';turnId:string}|{type:'silence';durationMs:number})[];voices:Record<string,string>;sourceRefs:{blockId:string;page:number}[];state:string;error?:string;assetId?:string;durationMs?:number;job?:{done:number;total:number}};
export const roleNames: Record<SourceRole, string> = {
  questions: "题目",
  answers: "答案",
  explanations: "解析",
  essays: "范文",
  transcript: "听力原文",
  audio: "音频",
  mixed: "混合资料",
};
export const sectionNames: Record<Section, string> = {
  reading: "阅读",
  listening: "听力",
  writing: "写作",
  speaking: "口语",
};
