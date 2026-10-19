import { useEffect, useState } from "react";
import type {
  Mode,
  Section,
  PaperVersion,
  QuestionKind,
} from "../domain/paper";
import type { Scope } from "../domain/session";
import type { DesktopPort, PaperSummary } from "../platform/desktop";
import { errorText } from "../platform/desktop";
import { sectionLabels } from "./Home";
import { checkReadiness } from "../domain/readiness";
import { questionLabels } from "../exam/questionRegistry";
import {CloudLibrary} from "../cloud/CloudLibrary";
export function Library({
  mode,
  port,
  onStart,
  onSettings,
}: {
  mode: Mode;
  port: DesktopPort | null;
  onStart: (p: PaperVersion, s: Section, scope: Scope) => Promise<void>;
  onSettings: () => void;
}) {
  const [section, setSection] = useState<Section>("reading");
  const [papers, setPapers] = useState<PaperSummary[]>([]);
  const [part, setPart] = useState("");
  const [kind, setKind] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const refresh = () => {
    if (port)
      port
        .listPapers()
        .then(setPapers)
        .catch((e) => setError(errorText(e)));
  };
  useEffect(refresh, [port]);
  async function start(item: PaperSummary) {
    if (!port) return;
    setBusy(true);
    setError("");
    try {
      const p = await port.loadPaper(item.id, item.version);
      const selected = mode === "exam" ? "listening" : section;
      const issues = checkReadiness(p, mode, selected);
      if (issues.length) throw Error(issues.map((x) => x.message).join("；"));
      if(mode==="exam"){
        const [config,keys]=await Promise.all([port.getConfig(),port.credentialStatus()]);
        for(const id of ["deepseek","qwenAsr","qwenTts"] as const){if(!config.providers[id].enabled||!keys[id])throw Error(`整套考试需要 ${id}，请先在设置中配置并测试`);}
      }
      await onStart(p, selected, mode==="exam"?{}:{
        part: part ? Number(part) : undefined,
        kind: kind ? (kind as QuestionKind) : undefined,
      });
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  }
  return (
    <>
      <div className="eyebrow">
        {mode === "exam" ? "FULL ACADEMIC TEST" : "FOCUSED PRACTICE"}
      </div>
      <h1>{mode === "exam" ? "选择完整模拟套卷" : "选择练习"}</h1>
      <p className="muted">
        {mode === "exam"
          ? "整套校验完成后才能开考。开发验证卷不用于正式模拟。"
          : "选一个科目，在独立房间开始本次练习。"}
      </p>
      {mode === "training" && (
        <>
          <div className="section-tabs">
            {Object.entries(sectionLabels).map(([id, label]) => (
              <button
                className={section === id ? "selected" : ""}
                key={id}
                onClick={() => {
                  setSection(id as Section);
                  setPart("");
                  setKind("");
                  setError("");
                }}
              >
                {label}
              </button>
            ))}
          </div>
          <div className="filter-row">
            <label>
              练习范围{" "}
              <select
                aria-label="练习部分"
                value={part}
                onChange={(e) => setPart(e.target.value)}
              >
                <option value="">整项</option>
                {Array.from(
                  {
                    length:
                      section === "listening"
                        ? 4
                        : section === "writing"
                          ? 2
                          : 3,
                  },
                  (_, i) => (
                    <option key={i} value={i + 1}>
                      {section === "writing" ? "Task" : "Part"} {i + 1}
                    </option>
                  ),
                )}
              </select>
            </label>
            {["reading", "listening"].includes(section) && (
              <label>
                题型{" "}
                <select
                  aria-label="练习题型"
                  value={kind}
                  onChange={(e) => setKind(e.target.value)}
                >
                  <option value="">全部题型</option>
                  {Object.entries(questionLabels)
                    .filter(
                      ([k]) => !["writing_task", "speaking_prompt"].includes(k),
                    )
                    .map(([id, name]) => (
                      <option key={id} value={id}>
                        {name}
                      </option>
                    ))}
                </select>
              </label>
            )}
          </div>
        </>
      )}
      <div className="library-actions">
        <button
          disabled={!port || busy}
          onClick={async () => {
            try {
              await port?.importPaper();
              refresh();
            } catch (e) {
              setError(errorText(e));
            }
          }}
        >
          导入本地 JSON 卷包
        </button>
      </div>
      {port&&<CloudLibrary port={port} mode={mode} section={section} part={part} kind={kind} onPublished={refresh}/>}
      {error && (
        <p className="error" role="alert">
          {error}
        </p>
      )}
      {!port && (
        <div className="notice">
          请先设置资产库和学习记录目录。
          <button onClick={onSettings}>前往设置</button>
        </div>
      )}
      <div className="paper-list">
        {papers
          .filter((p) => mode === "exam" || p.sections.includes(section))
          .map((p) => (
            <article className="paper-card" key={`${p.id}-${p.version}`}>
              <div>
                <span className="pill">
                  {p.provenance.kind === "developer_fixture"
                    ? "开发验证卷"
                    : p.provenance.kind === "ai_generated"
                      ? "AI 生成卷"
                      : "导入资料"}
                </span>
                <h3>{p.title}</h3>
                <p>
                  {p.sections.map((s) => sectionLabels[s]).join(" / ")}{" "}
                  <span>· 版本 {p.version}</span>
                </p>
                <small>{p.provenance.source}</small>
              </div>
              <button
                className="primary"
                disabled={busy}
                onClick={() => start(p)}
              >
                {busy ? "校验中…" : "进入考场"}
              </button>
            </article>
          ))}
      </div>
      {port &&
        papers.filter((p) => mode === "exam" || p.sections.includes(section))
          .length === 0 && (
          <div className="empty-state">
            <h3>还没有这部分的试卷</h3>
            <p>导入带有答案、依据和所需媒体的 JSON 卷包后开始。</p>
          </div>
        )}
    </>
  );
}
