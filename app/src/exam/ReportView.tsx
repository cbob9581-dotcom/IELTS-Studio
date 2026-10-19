import {RecordingPlayback} from "../speaking/RecordingPlayback";
import type { PaperVersion } from "../domain/paper";
import type { AttemptSnapshot } from "../domain/session";
import type { ObjectiveResult } from "../domain/grading";
import type {DesktopPort} from "../platform/desktop";
import {AssessmentPanel} from "../cloud/AssessmentPanel";
export function ReportView({
  paper,
  attempt,
  result,
  port=null,
}: {
  paper: PaperVersion;
  attempt: AttemptSnapshot;
  result: ObjectiveResult;
  port?:DesktopPort|null;
}) {
  const section = paper.sections.find((s) => s.section === attempt.section)!;
  return (
    <>
      <div className="eyebrow">SESSION COMPLETE</div>
      <h1>练习报告</h1>
      <p className="muted">
        {paper.title} · 版本 {paper.version}
      </p>
      <div className="report-score">
        <span>{attempt.section === "speaking" ? "口语记录" :result.total?"客观题独立作答得分":attempt.section==="writing"?"写作记录":"练习记录"}</span>
        <strong>
          {result.total ? `${result.score} / ${result.total}` : "已提交"}
        </strong>
        <p>{result.total?"客观题显示原始得分；AI 模拟卷未做题目难度标定，不作固定 Band 换算。":"云端评测与具体反馈见下方。"}</p>
      </div>
      {["writing","speaking"].includes(attempt.section)&&<AssessmentPanel port={port} attemptId={attempt.id}/>}
      {paper.provenance.kind === "developer_fixture" && (
        <p className="notice">
          开发验证卷：本次仅验证练习流程，不代表正式 IELTS 成绩。
        </p>
      )}
      {attempt.interruptionEvents.length > 0 && (
        <p className="notice">
          本次记录包含 {attempt.interruptionEvents.length} 次中断。
        </p>
      )}
      {!!attempt.revealedQuestions?.length&&<p className="notice">有 {attempt.revealedQuestions.length} 道题查看过答案，已从独立作答总分中排除。</p>}
      <div className="report-items">
        {attempt.section==="speaking" ? <>
          {(attempt.speaking?.turns??[]).map(turn=><article className="settings-card" key={turn.turnId}>
            <h3>{turn.prompt}</h3><p>原始识别：{turn.text}</p><small>录音：{turn.recordingId}.wav</small><RecordingPlayback port={port} attemptId={attempt.id} takeId={turn.recordingId}/>
          </article>)}
          {attempt.speakingNotes?.text&&<article className="settings-card"><h3>草稿纸笔记</h3><p>{attempt.speakingNotes.text}</p></article>}
        </> : result.items.map((item) => {
          const q = section.questions.find((q) => q.id === item.questionId)!;
          const response = attempt.responses.find(
            (x) => x.questionId === q.id,
          )?.value;
          return (
            <article className="settings-card" key={q.id}>
              <span
                className={`pill ${item.correct === false ? "incorrect" : ""}`}
              >
                第 {q.number} 题 ·{" "}
                {attempt.revealedQuestions?.includes(q.id)?"已查看答案，不计独立得分":item.correct === null
                  ? "主观题"
                  : item.correct
                    ? "正确"
                    : item.score>0?`部分正确 ${item.score}/${item.max}`:"未得分"}
              </span>
              <h3>{q.prompt}</h3>
              <p>
                你的作答：
                {typeof response === "string"
                  ? response
                  : response
                    ? JSON.stringify(response)
                    : "未作答"}
              </p>
              {q.reviewMaterials?.map(material=><details key={material.id}><summary>{material.kind==="essays"?"原资料范文":material.kind==="transcript"?"听力原文":"原资料解析"} · 来源第 {material.anchor.page??"未知"} 页</summary><p style={{whiteSpace:"pre-wrap"}}>{material.text}</p><small>资料来源：{material.anchor.documentId} · {material.anchor.blockIds.join(", ")}</small></details>)}
              {q.answer && (
                <>
                  <p>
                    参考答案：
                    {q.answer.accepted
                      .map((x) =>
                        typeof x === "string" ? x : JSON.stringify(x),
                      )
                      .join(" / ")}
                  </p>
                  <small className="muted">依据：{q.evidence.join("；")}</small>
                </>
              )}
            </article>
          );
        })}
      </div>
    </>
  );
}
