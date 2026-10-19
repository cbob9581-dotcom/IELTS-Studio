import {
  validatePaper,
  type PaperVersion,
  type Mode,
  type Section,
  type ValidationIssue,
} from "./paper";
export function checkReadiness(
  p: PaperVersion,
  mode: Mode,
  section?: Section,
): ValidationIssue[] {
  const issues = validatePaper(p);
  const add = (code: string, message: string) =>
    issues.push({ path: "readiness", code, message });
  if (issues.length) return issues;
  if (mode === "exam" && p.provenance.kind === "developer_fixture")
    add("developer_fixture", "开发验证卷不可用于完整模拟考试");
  if (p.status !== "ready") add("not_ready", "试卷尚未发布");
  const selected =
    mode === "exam"
      ? p.sections
      : p.sections.filter((s) => s.section === section);
  if (!selected.length) add("missing_section", "卷中没有所选科目");
  if (mode === "exam" && new Set(p.sections.map((s) => s.section)).size !== 4)
    add("incomplete_suite", "考试需要四项完整套卷");
  for (const s of selected) {
    if (!s.timingSource.trim()) add("missing_timing", "缺少计时规则来源");
    if (s.section === "listening" && !s.audioAssetIds?.length)
      add("missing_audio", "听力缺少音频");
    if (mode === "exam") {
      const counts = {
        listening: [40, 4],
        reading: [40, 3],
        writing: [2, 2],
        speaking: [0, 3],
      }[s.section];
      const parts = [...new Set(s.questions.map((q) => q.part))].sort(
        (a, b) => a - b,
      );
      const numbers = s.questions.flatMap(q=>Array.from({length:["reading","listening"].includes(s.section)&&q.kind==="multiple_choice"?q.points:1},(_,i)=>q.number+i)).sort((a,b)=>a-b);
      if (
        (counts[0] && numbers.length !== counts[0]) ||
        parts.length !== counts[1] ||
        parts.some((part, i) => part !== i + 1) ||
        numbers.some((number, i) => number !== i + 1)
      )
        add("incomplete_structure", `${s.section} 题数、编号或部分不完整`);
      if (
        s.section === "reading" &&
        new Set(s.paragraphs.map((x) => x.part)).size !== 3
      )
        add("incomplete_passages", "阅读需要三篇文章");
      if (
        s.section === "writing" &&
        s.questions.some((q) => q.kind !== "writing_task")
      )
        add("invalid_task", "写作题型不正确");
      if (
        s.section === "speaking" &&
        s.questions.some((q) => q.kind !== "speaking_prompt")
      )
        add("invalid_task", "口语题型不正确");
    }
  }
  return issues;
}
