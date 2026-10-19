import type { Question, ResponseValue } from "../domain/paper";
import { WritingEditor } from "./WritingEditor";
export function QuestionRenderer({
  question: q,
  value,
  onChange,
  imageUrl,
}: {
  question: Question;
  value: ResponseValue;
  onChange: (v: ResponseValue) => void;
  imageUrl?: string;
}) {
  if (q.kind === "writing_task")
    return (
      <>
        {q.imageAssetId &&
          (imageUrl ? (
            <img
              className="writing-chart"
              src={imageUrl}
              alt="写作 Task 1 图表"
            />
          ) : (
            <p>正在加载图表…</p>
          ))}
        <WritingEditor
          value={typeof value === "string" ? value : ""}
          onChange={onChange}
          minWords={q.minWords}
        />
      </>
    );
  if (q.kind === "speaking_prompt")
    return (
      <div className="notice">
        口语在三维考场进行，请回到房间与考官互动。
      </div>
    );
  const options =
    q.kind === "true_false_not_given"
      ? ["TRUE", "FALSE", "NOT GIVEN"]
      : q.kind === "yes_no_not_given"
        ? ["YES", "NO", "NOT GIVEN"]
        : (q.options ?? []);
  if (
    ["single_choice", "true_false_not_given", "yes_no_not_given"].includes(
      q.kind,
    )
  )
    return (
      <div className="answer-options">
        {options.map((o) => (
          <label key={o}>
            <input
              type="radio"
              name={q.id}
              checked={value === o}
              onChange={() => onChange(o)}
            />
            {o}
          </label>
        ))}
      </div>
    );
  if (q.kind === "multiple_choice")
    return (
      <div className="answer-options">
        {options.map((o) => (
          <label key={o}>
            <input
              type="checkbox"
              checked={Array.isArray(value) && value.includes(o)}
              onChange={(e) => {
                const old = Array.isArray(value) ? value : [];
                onChange(
                  e.target.checked ? [...old, o] : old.filter((x) => x !== o),
                );
              }}
            />
            {o}
          </label>
        ))}
      </div>
    );
  if (q.kind === "matching") {
    const record =
      typeof value === "object" && !Array.isArray(value) ? value : {};
    return (
      <div>
        {q.slots?.map((slot) => (
          <label className="match-row" key={slot}>
            {slot}
            <select
              aria-label={slot}
              value={record[slot] ?? ""}
              onChange={(e) => onChange({ ...record, [slot]: e.target.value })}
            >
              <option value="">请选择</option>
              {options.map((o) => (
                <option key={o}>{o}</option>
              ))}
            </select>
          </label>
        ))}
      </div>
    );
  }
  if (q.kind === "heading_match")
    return (
      <select
        aria-label={`第 ${q.number} 题答案`}
        value={typeof value === "string" ? value : ""}
        onChange={(e) => onChange(e.target.value)}
      >
        <option value="">请选择标题</option>
        {options.map((o) => (
          <option key={o}>{o}</option>
        ))}
      </select>
    );
  if (q.kind === "diagram_label") {
    const record =
      typeof value === "object" && !Array.isArray(value) ? value : {};
    return (
      <div>
        <div className="diagram">
          {imageUrl ? (
            <img src={imageUrl} alt="题目标注图" />
          ) : (
            <p>正在加载标注图…</p>
          )}
          {q.labels?.map((label) => (
            <span
              className="diagram-marker"
              key={label.id}
              style={{ left: `${label.x * 100}%`, top: `${label.y * 100}%` }}
            >
              {label.id}
            </span>
          ))}
        </div>
        {q.labels?.map((label) => (
          <label className="match-row" key={label.id}>
            {label.id}
            <input
              aria-label={`标注 ${label.id}`}
              value={record[label.id] ?? ""}
              onChange={(e) =>
                onChange({ ...record, [label.id]: e.target.value })
              }
            />
          </label>
        ))}
      </div>
    );
  }
  return (
    <div>
      {!!q.options?.length&&<p className="notice">{q.options.join("　 /　 ")}</p>}
      <input
        className="short-answer"
        aria-label={`第 ${q.number} 题答案`}
        value={typeof value === "string" ? value : ""}
        onChange={(e) => onChange(e.target.value)}
        autoComplete="off"
        spellCheck={false}
      />
      {q.answer?.maxWords && (
        <small className="muted">不超过 {q.answer.maxWords} 个词</small>
      )}
    </div>
  );
}
