import { it, expect } from "vitest";
import { gradeObjective } from "../domain/grading";
import type { SectionPaper } from "../domain/paper";
it("excludes revealed answers from independent practice totals", () => {
  const p = {
    questions: [
      {
        id: "q",
        kind: "gap_fill",
        points: 1,
        answer: { accepted: ["A"], caseSensitive: false },
      },
    ],
  } as SectionPaper;
  expect(
    gradeObjective(p, [{ questionId: "q", value: "A" }], ["q"]).total,
  ).toBe(0);
});
