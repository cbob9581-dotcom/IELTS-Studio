import { useEffect, useRef, useState } from "react";
import type { PaperVersion, ResponseValue } from "../domain/paper";
import type { AttemptSnapshot, Annotation } from "../domain/session";
import { selectScope } from "../domain/session";
import type { DesktopPort } from "../platform/desktop";
import { errorText } from "../platform/desktop";
import { QuestionRenderer } from "./QuestionRenderer";
import { questionLabels } from "./questionRegistry";
import { ListeningPlayer } from "./ListeningPlayer";
import { selectedAnnotation } from "./annotations";
function MarkedText({ text, marks }: { text: string; marks: Annotation[] }) {
  const points = [
    ...new Set([0, text.length, ...marks.flatMap((a) => [a.start, a.end])]),
  ]
    .filter((x) => x >= 0 && x <= text.length)
    .sort((a, b) => a - b);
  return (
    <>
      {points.slice(0, -1).map((start, i) => {
        const end = points[i + 1];
        return marks.some((m) => m.start < end && m.end > start) ? (
          <mark key={start}>{text.slice(start, end)}</mark>
        ) : (
          <span key={start}>{text.slice(start, end)}</span>
        );
      })}
    </>
  );
}
export function ExamView({
  paper,
  attempt,
  port,
  onChange,
  onSubmit,
  paused,
  onPause,
  masterVolume = 0.75,
  outputDeviceId = "",
}: {
  paper: PaperVersion;
  attempt: AttemptSnapshot;
  port: DesktopPort;
  onChange: (a: AttemptSnapshot) => void;
  onSubmit: () => void;
  paused: boolean;
  onPause: () => void;
  masterVolume?: number;
  outputDeviceId?: string;
}) {
  const section = selectScope(
    paper.sections.find((s) => s.section === attempt.section)!,
    attempt.scope,
  );
  const [current, setCurrent] = useState(section.questions[0].id);
  const isReading=attempt.section==='reading';
  const readingParts=[...new Set(section.questions.map(q=>q.part))].sort((a,b)=>a-b);
  const [readingPart,setReadingPart]=useState(readingParts[0]);
  const selectedPart=readingParts.includes(readingPart)?readingPart:readingParts[0];
  const paragraphs=isReading?section.paragraphs.filter(p=>p.part===selectedPart):section.paragraphs;
  const [showAnswers, setShowAnswers] = useState(false);
  const [urls, setUrls] = useState<Record<string, string>>({});
  const [mediaError, setMediaError] = useState("");
  const [audioReady, setAudioReady] = useState(false);
  const [note, setNote] = useState("");
  const [selection, setSelection] = useState("");
  const range = useRef<{ range: Range; paragraph: HTMLElement } | null>(null);
  function changeReadingPart(part:number){setReadingPart(part);setShowAnswers(false);setSelection('');setNote('');range.current=null;}
  useEffect(() => {
    let cancel = false;
    const created: string[] = [];
    const refs = new Set([
      ...(section.audioAssetIds ?? []),
      ...section.questions.flatMap((q) =>
        [...(q.imageAssetId ? [q.imageAssetId] : []),...(q.imageAssetIds??[])],
      ),
    ]);
    Promise.all(
      [...refs].map(async (id) => {
        const a = paper.assets.find((a) => a.id === id);
        if (!a) throw Error(`缺少资源 ${id}`);
        const bytes = port.readAssetBinary?await port.readAssetBinary(paper.id,paper.version,id):new Uint8Array(await port.readAsset(paper.id,paper.version,id)).buffer;
        if (cancel) return [id, ""] as const;
        const url = URL.createObjectURL(
          new Blob([bytes], { type: a.mediaType }),
        );
        created.push(url);
        return [id, url] as const;
      }),
    )
      .then((entries) => {
        if (!cancel) setUrls(Object.fromEntries(entries));
      })
      .catch((e) => {
        if (!cancel) setMediaError(errorText(e));
      });
    return () => {
      cancel = true;
      created.forEach(URL.revokeObjectURL);
    };
  }, [paper, port, attempt.section]);
  const update = (id: string, value: ResponseValue) =>
    onChange({
      ...attempt,
      responses: [
        ...attempt.responses.filter((r) => r.questionId !== id),
        { questionId: id, value },
      ],
    });
  const flag = (id: string) =>
    onChange({
      ...attempt,
      flagged: attempt.flagged.includes(id)
        ? attempt.flagged.filter((x) => x !== id)
        : [...attempt.flagged, id],
    });
  const addAnnotation = () => {
    if (!range.current) return;
    const a = selectedAnnotation(
      range.current.paragraph,
      range.current.range,
      note,
    );
    if (a) onChange({ ...attempt, annotations: [...attempt.annotations, a] });
    setSelection("");
    setNote("");
    window.getSelection()?.removeAllRanges();
  };
  const isWriting = attempt.section === "writing";
  const questions = isWriting
    ? section.questions.filter((q) => q.id === current)
    : isReading?section.questions.filter(q=>q.part===selectedPart):section.questions;
  const listening = attempt.section === "listening";
  const blocked = paused || !!mediaError || (listening && !audioReady);
  return (
    <div className="exam">
      <div className="exam-top">
        <div>
          <h2>{section.title}</h2>
          <span className="muted">
            {attempt.mode === "exam" ? "考试模式" : "训练模式"} ·{" "}
            {paper.provenance.kind === "developer_fixture"
              ? "开发验证卷"
              : "Academic"}
          </span>
        </div>
        <div className="exam-actions">
          {attempt.mode === "training" && (
            <>
              <button onClick={onPause}>
                {paused ? "继续计时" : "暂停练习"}
              </button>
              <button onClick={() => {if(!showAnswers)onChange({...attempt,revealedQuestions:[...new Set([...(attempt.revealedQuestions??[]),...questions.map(q=>q.id)])]});setShowAnswers(!showAnswers);}}>
                {showAnswers ? "收起解析" : "查看解析"}
              </button>
            </>
          )}
          <button className="primary" onClick={onSubmit}>
            提交作答
          </button>
        </div>
      </div>
      {isReading&&<nav className="library-actions" aria-label="阅读文章切换">{readingParts.map(part=>{const qs=section.questions.filter(q=>q.part===part);const first=qs[0]?.number;const last=Math.max(...qs.map(q=>q.number+q.points-1));return <button key={part} aria-pressed={part===selectedPart} onClick={()=>{changeReadingPart(part);setCurrent(qs[0].id);}}>第 {part} 篇 · 第 {first}–{last} 题</button>;})}<span className="muted">本次阅读共 {readingParts.length} 篇 · {section.questions.reduce((n,q)=>n+q.points,0)} 题</span></nav>}
      {listening && (
        <ListeningPlayer
          ranges={section.audioPlaylist}
          urls={(section.audioAssetIds ?? [])
            .map((id) => urls[id])
            .filter(Boolean)}
          position={attempt.listening}
          onPosition={listening=>onChange({...attempt,listening})}
          mode={attempt.mode}
          initialVolume={masterVolume}
          outputDeviceId={outputDeviceId}
          paused={paused}
          onReady={() => {setAudioReady(true);setMediaError("");}}
          onError={setMediaError}
        />
      )}
      {listening&&section.audioPlaylist&&<p className="muted">音频来源：{[...new Set(section.audioPlaylist.map(t=>t.origin==='book'?'真题脚本合成':t.origin==='user'?'用户文本合成':'用户上传'))].join('、')}</p>}
      {listening&&attempt.mode==='training'&&!!section.audioScripts?.length&&<details><summary>查看听力稿（训练模式）</summary>{section.audioScripts.filter(s=>!attempt.scope?.part||s.part===attempt.scope.part).map((s,i)=><div key={i}><p>Part {s.part} · {s.origin==='book'?'真题脚本合成':'用户文本合成'}</p>{s.turns.map(t=><p key={t.id}><strong>{t.speaker}: </strong>{t.text}</p>)}</div>)}</details>}
      {mediaError && (
        <p className="error" role="alert">
          {mediaError}
        </p>
      )}
      {paused && <p className="notice">练习已暂停。继续后可作答。</p>}
      <div
        className={`exam-columns ${section.paragraphs.length ? "" : "no-passage"}`}
      >
        <div className="passage-pane">
          {section.paragraphs.length > 0 && (
            <>
              <div className="eyebrow">{isReading?`READING PASSAGE ${selectedPart}`:'试题材料'}</div>
              <div
                onMouseUp={() => {
                  const s = window.getSelection();
                  if (!s?.rangeCount || s.isCollapsed) return;
                  const r = s.getRangeAt(0);
                  const node =
                    r.startContainer.nodeType === Node.TEXT_NODE
                      ? r.startContainer.parentElement
                      : (r.startContainer as HTMLElement);
                  const paragraph =
                    node?.closest<HTMLElement>("[data-paragraph]");
                  if (paragraph && paragraph.contains(r.endContainer)) {
                    range.current = { range: r.cloneRange(), paragraph };
                    setSelection(r.toString());
                  }
                }}
              >
                {paragraphs.map((p) => (
                  <p data-paragraph={p.id} key={p.id}>
                    <MarkedText
                      text={p.text}
                      marks={attempt.annotations.filter(
                        (a) => a.paragraphId === p.id,
                      )}
                    />
                  </p>
                ))}
              </div>
              {selection && (
                <div className="annotation-toolbar">
                  <small>选中：{selection.slice(0, 50)}</small>
                  <input
                    aria-label="选中文字笔记"
                    value={note}
                    onChange={(e) => setNote(e.target.value)}
                    placeholder="添加笔记（可选）"
                  />
                  <button onClick={addAnnotation} disabled={blocked}>
                    高亮并保存
                  </button>
                </div>
              )}
              {attempt.annotations.length > 0 && (
                <details open>
                  <summary>
                    我的笔记与高亮 ({attempt.annotations.length})
                  </summary>
                  {attempt.annotations.map((a) => (
                    <div className="annotation" key={a.id}>
                      <mark>{a.text}</mark>
                      <p>{a.note}</p>
                      <button
                        disabled={blocked}
                        onClick={() =>
                          onChange({
                            ...attempt,
                            annotations: attempt.annotations.filter(
                              (x) => x.id !== a.id,
                            ),
                          })
                        }
                      >
                        移除
                      </button>
                    </div>
                  ))}
                </details>
              )}
            </>
          )}
        </div>
        <div className="questions-pane">
          <div className="question-nav" aria-label="题号导航">
            {section.questions.map((q) => (
              <button
                className={`${attempt.responses.some((r) => r.questionId === q.id && JSON.stringify(r.value) !== '""') ? "answered" : ""} ${current === q.id ? "current" : ""} ${attempt.flagged.includes(q.id) ? "flagged" : ""}`}
                key={q.id}
                onClick={() => {
                  setCurrent(q.id);
                  if(isReading&&q.part!==selectedPart)changeReadingPart(q.part);
                  if (!isWriting)
                    requestAnimationFrame(()=>document
                      .getElementById(`q-${q.id}`)
                      ?.scrollIntoView?.({ block: "center", behavior: "smooth" }));
                }}
              >
                {q.number}
                {attempt.flagged.includes(q.id) ? " ⚑" : ""}
              </button>
            ))}
          </div>
          {questions.map((q) => (
            <article className="question-block" key={q.id} id={`q-${q.id}`}>
              <div className="question-heading">
                <span className="pill">
                  {isWriting ? "Task" : "Question"} {q.number} ·{" "}
                  {questionLabels[q.kind]}
                </span>
                <button
                  className="flag"
                  aria-label={`标记第 ${q.number} 题`}
                  onClick={() => flag(q.id)}
                  disabled={blocked}
                >
                  {attempt.flagged.includes(q.id) ? "⚑ 已标记" : "⚐ 标记"}
                </button>
              </div>
              <h3>{q.prompt}</h3>
              <fieldset disabled={blocked}>
                {q.imageAssetId&&!['writing_task','diagram_label'].includes(q.kind)&&urls[q.imageAssetId]&&<img className="writing-chart" src={urls[q.imageAssetId]} alt={`第 ${q.number} 题原始题图`}/>}
                {q.imageAssetIds?.filter(id=>id!==q.imageAssetId).map(id=><details open key={id}><summary>题图（可折叠）</summary>{urls[id]?<img className="writing-chart" src={urls[id]} alt={`第 ${q.number} 题原始题图`}/>:<p>正在加载题图…</p>}</details>)}
                <QuestionRenderer
                  question={q}
                  value={
                    attempt.responses.find((r) => r.questionId === q.id)
                      ?.value ?? ""
                  }
                  onChange={(value) => update(q.id, value)}
                  imageUrl={q.imageAssetId ? urls[q.imageAssetId] : undefined}
                />
              </fieldset>
              {showAnswers && attempt.mode === "training" && q.answer && (
                <div className="solution">
                  <b>参考答案</b>
                  <p>
                    {q.answer.accepted
                      .map((x) =>
                        typeof x === "string" ? x : JSON.stringify(x),
                      )
                      .join(" / ")}
                  </p>
                  <small>{q.evidence.join("；")}</small>
                </div>
              )}
            </article>
          ))}
        </div>
      </div>
    </div>
  );
}
