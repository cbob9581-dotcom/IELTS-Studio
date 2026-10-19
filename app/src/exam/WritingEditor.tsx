export function wordCount(text: string): number {
  return text.trim().split(/\s+/).filter(Boolean).length;
}
export function WritingEditor({
  value,
  onChange,
  minWords,
}: {
  value: string;
  onChange: (v: string) => void;
  minWords?: number;
}) {
  return (
    <div className="writing-editor">
      <textarea
        aria-label="写作答卷"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder="Write your response here…"
        spellCheck={false}
      />
      <div className="word-count">
        {wordCount(value)} words{" "}
        <span>{minWords ? `建议至少 ${minWords} 词` : ""}</span>
      </div>
    </div>
  );
}
