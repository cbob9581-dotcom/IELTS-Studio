export type SourceAnchor = {
  projectId?: string;
  documentId: string;
  blockIds: string[];
  page?: number | null;
  sha256?: string;
};
import validate from "./paper-validator.js";
export type Section = "listening" | "reading" | "writing" | "speaking";
export type Mode = "exam" | "training";
export const questionKinds = [
  "single_choice",
  "multiple_choice",
  "matching",
  "gap_fill",
  "short_answer",
  "diagram_label",
  "true_false_not_given",
  "yes_no_not_given",
  "heading_match",
  "writing_task",
  "speaking_prompt",
] as const;
export type QuestionKind = (typeof questionKinds)[number];
export type ResponseValue = string | string[] | Record<string, string>;
export type Response = { questionId: string; value: ResponseValue };
export type AnswerRule = {
  accepted: ResponseValue[];
  caseSensitive: boolean;
  maxWords?: number;
  maxNumbers?: number;
};
export type Question = {
  id: string;
  number: number;
  kind: QuestionKind;
  prompt: string;
  part: number;
  points: number;
  options?: string[];
  slots?: string[];
  imageAssetId?: string;
  imageAssetIds?: string[];
  sourceAnchors?: SourceAnchor[];
  reviewMaterials?: {
    id: string;
    kind: "explanations" | "essays" | "transcript";
    text: string;
    anchor: SourceAnchor;
  }[];
  labels?: { id: string; x: number; y: number }[];
  answer?: AnswerRule;
  evidence: string[];
  minWords?: number;
};
export type AssetRef = {
  id: string;
  path: string;
  sha256: string;
  mediaType: string;
};
export type Paragraph = { id: string; text: string; part: number };
export type AudioTrack={assetId:string;part:number|null;origin:'book'|'user'|'uploaded';durationMs?:number;segments:{part:number;startMs:number;endMs:number}[];startMs?:number;endMs?:number};
export type SectionPaper = {
  section: Section;
  title: string;
  durationMs: number;
  timingSource: string;
  questions: Question[];
  paragraphs: Paragraph[];
  audioAssetIds?: string[];
  audioParts?: Record<string, number>;
  audioPlaylist?:AudioTrack[];
  audioScripts?:{part:number;origin:'book'|'user';turns:{id:string;speaker:string;text:string}[]}[];
};
export type PaperVersion = {
  schemaVersion: 1;
  id: string;
  version: number;
  title: string;
  testType: "academic";
  status: "draft" | "validating" | "ready" | "review_required" | "failed";
  provenance: {
    kind: "developer_fixture" | "imported" | "ai_generated";
    source: string;
  };
  sections: SectionPaper[];
  assets: AssetRef[];
};
export type ValidationIssue = { path: string; code: string; message: string };
export function validatePaper(input: unknown): ValidationIssue[] {
  if (!validate(input))
    return (validate.errors ?? []).map((e) => ({
      path: e.instancePath,
      code: "schema",
      message: e.message ?? "卷格式错误",
    }));
  const p = input as PaperVersion;
  const issues: ValidationIssue[] = [];
  const issue = (path: string, code: string, message: string) =>
    issues.push({ path, code, message });
  const ids = new Set<string>();
  const assetIds = new Set<string>();
  for (const a of p.assets) {
    if (assetIds.has(a.id))
      issue(`assets.${a.id}`, "duplicate_asset", "资源 ID 重复");
    assetIds.add(a.id);
    if (!isSafePath(a.path))
      issue(`assets.${a.id}`, "unsafe_path", "资源路径必须在卷包内");
  }
  const sections = new Set<Section>();
  for (const s of p.sections) {
    if (sections.has(s.section))
      issue(s.section, "duplicate_section", "科目重复");
    sections.add(s.section);
    const numbers = new Set<number>();
    for (const q of s.questions) {
      const path = `${s.section}.${q.id}`;
      if (ids.has(q.id)) issue(path, "duplicate_id", "题目 ID 重复");
      ids.add(q.id);
      if (numbers.has(q.number)) issue(path, "duplicate_number", "题号重复");
      numbers.add(q.number);
      if (
        !["writing_task", "speaking_prompt"].includes(q.kind) &&
        (!q.answer?.accepted.length || !q.evidence.length)
      )
        issue(path, "missing_evidence", "客观题必须有答案和证据");
      if (
        [
          "single_choice",
          "multiple_choice",
          "matching",
          "heading_match",
        ].includes(q.kind) &&
        !q.options?.length
      )
        issue(path, "missing_options", "缺少选项");
      if (q.kind === "matching" && !q.slots?.length)
        issue(path, "missing_slots", "匹配题缺少项目");
      if (q.kind === "diagram_label" && (!q.imageAssetId || !q.labels?.length))
        issue(path, "missing_diagram", "标注题缺图片或坐标");
      for (const id of q.imageAssetIds ?? [])
        if (!assetIds.has(id)) issue(path, "missing_asset", "缺少图片资源");
      if (q.imageAssetId && !assetIds.has(q.imageAssetId))
        issue(path, "missing_asset", "缺少图片资源");
    }
    for (const [id, part] of Object.entries(s.audioParts ?? {}))
      if (!s.audioAssetIds?.includes(id) || part < 1 || part > 4)
        issue(s.section, "audio_mapping", "音频 Part 对应关系无效");
    for (const id of s.audioAssetIds ?? [])
      if (!assetIds.has(id)) issue(s.section, "missing_asset", "缺少音频资源");
    if(s.audioPlaylist){
      if(JSON.stringify(s.audioPlaylist.map(t=>t.assetId))!==JSON.stringify(s.audioAssetIds??[])||new Set(s.audioPlaylist.map(t=>t.assetId)).size!==s.audioPlaylist.length)issue(s.section,'audio_timeline','音频播放计划与资产列表顺序不一致或重复');
      for(const t of s.audioPlaylist){let previous=0;const seen=new Set<number>();for(const range of t.segments){if(range.endMs<=range.startMs||range.startMs<previous||seen.has(range.part)||(t.durationMs!==undefined&&range.endMs>t.durationMs))issue(s.section,'audio_timeline','Part 时间段重叠、倒置或超过音频时长');seen.add(range.part);previous=range.endMs;}}
    }
  }
  return issues;
}
export function isSafePath(path: string): boolean {
  return (
    !!path &&
    !/[\\:\x00-\x1f]/.test(path) &&
    !path.startsWith("/") &&
    path.split("/").every((x) => x !== "" && x !== "." && x !== "..")
  );
}
