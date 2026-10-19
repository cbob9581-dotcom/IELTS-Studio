import { useEffect, useState } from "react";
import type { DesktopPort } from "../platform/desktop";
import { errorText } from "../platform/desktop";
import type { AttemptSnapshot } from "../domain/session";
import { sectionLabels } from "./Home";
export function History({
  port,
  onResume,
}: {
  port: DesktopPort | null;
  onResume: (a: AttemptSnapshot) => Promise<void>;
}) {
  const [items, setItems] = useState<AttemptSnapshot[]>([]);
  const [titles,setTitles]=useState<Record<string,string>>({}),[filter,setFilter]=useState("");
  const [error, setError] = useState("");
  useEffect(()=>{port?.listPapers?.().then(ps=>setTitles(Object.fromEntries(ps.map(p=>[p.id,p.title])))).catch(()=>{});},[port]);
  useEffect(() => {
    port
      ?.listAttempts()
      .then(setItems)
      .catch((e) => setError(errorText(e)));
  }, [port]);
  return (
    <>
      <div className="eyebrow">YOUR LEARNING JOURNEY</div>
      <h1>学习记录</h1>
      <p className="muted">继续未完成的练习，回看已经走过的每一步。</p>
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}
      {!items.length && (
        <div className="empty-state">开始一次练习后，记录会保存在这里。</div>
      )}
      <label>搜索学习记录<input value={filter} onChange={e=>setFilter(e.target.value)} placeholder="试卷名称、科目或套卷编号"/></label>
      {items.filter(a=>`${titles[a.paperId]??a.paperId} ${sectionLabels[a.section]} ${a.examRunId??""}`.toLowerCase().includes(filter.toLowerCase())).map((a) => (
        <article className="paper-card" key={a.id}>
          <div>
            <span className="pill">
              {a.status === "submitted" ? "已提交" : "未完成"} ·{" "}
              {a.mode === "exam" ? "考试" : "训练"}
            </span>
            <h3>
              {sectionLabels[a.section]} · {titles[a.paperId]??a.paperId}
            </h3>
            <p>
              {new Date(
                a.updatedAt ?? a.clock.lastObservedUtcMs,
              ).toLocaleString("zh-CN")}{" "}
              · 已保存 {a.section==="speaking"?`${a.speaking?.turns.length??0} 轮回答`:`${a.responses.length} 道作答`}
            </p>
            {a.examRunId&&<small>套卷 {a.examRunId.slice(0,8)} · 已提交 {items.filter(x=>x.examRunId===a.examRunId&&x.status==="submitted").length}/4 科</small>}
            {a.interruptionEvents.length > 0 && (
              <small>记录了 {a.interruptionEvents.length} 次中断</small>
            )}
          </div>
          <button
            onClick={() => onResume(a).catch((e) => setError(errorText(e)))}
          >
            {a.status === "submitted" ? "查看报告" : "继续练习"}
          </button>
        </article>
      ))}
    </>
  );
}
