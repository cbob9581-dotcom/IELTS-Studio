import type { Annotation } from "../domain/session";
export function selectedAnnotation(
  paragraph: HTMLElement,
  range: Range,
  note: string,
): Annotation | null {
  if (
    !paragraph.contains(range.startContainer) ||
    !paragraph.contains(range.endContainer) ||
    range.collapsed
  )
    return null;
  const before = range.cloneRange();
  before.selectNodeContents(paragraph);
  before.setEnd(range.startContainer, range.startOffset);
  const start = before.toString().length;
  return {
    id: crypto.randomUUID(),
    paragraphId: paragraph.dataset.paragraph!,
    start,
    end: start + range.toString().length,
    text: range.toString(),
    note,
  };
}
