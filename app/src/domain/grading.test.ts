import { test, expect } from "vitest";
import { fixturePaper } from "../../test/fixtures/paper";
import { gradeObjective } from "./grading";
test("accepts declared variants only and enforces word limits", () => {
  const section = fixturePaper().sections[0];
  expect(
    gradeObjective(section, [{ questionId: "r1", value: "  SOLAR  " }]).score,
  ).toBe(1);
  expect(
    gradeObjective(section, [{ questionId: "r1", value: "sunlight" }]).score,
  ).toBe(0);
  expect(
    gradeObjective(section, [{ questionId: "r1", value: "the solar" }]).score,
  ).toBe(0);
});
test("multiple choice grades exact sets without partial guessing", () => {
  const section = fixturePaper().sections[0];
  section.questions[0] = {
    ...section.questions[0],
    kind: "multiple_choice",
    options: ["A", "B", "C"],
    answer: { accepted: [["A", "C"]], caseSensitive: false },
  };
  expect(
    gradeObjective(section, [{ questionId: "r1", value: ["C", "A"] }]).score,
  ).toBe(1);
  expect(
    gradeObjective(section, [{ questionId: "r1", value: ["A", "B", "C"] }])
      .score,
  ).toBe(0);
});
test("numbered multiple-choice group awards one mark per correct selection without over-selection",()=>{
 const s=fixturePaper().sections[0];s.questions=[{...s.questions[0],kind:"multiple_choice",points:2,options:["A","B","C","D"],answer:{accepted:[["A","C"]],caseSensitive:false}}];
 expect(gradeObjective(s,[{questionId:"r1",value:["A","B"]}]).score).toBe(1);
 expect(gradeObjective(s,[{questionId:"r1",value:["A","C"]}]).score).toBe(2);
 expect(gradeObjective(s,[{questionId:"r1",value:["A","B","C"]}]).score).toBe(0);
});
test("ONE WORD AND A NUMBER counts the number separately from the word limit",()=>{
 const s=fixturePaper().sections[0];s.questions=[{...s.questions[0],answer:{accepted:["12 October"],caseSensitive:false,maxWords:1,maxNumbers:1}}];
 expect(gradeObjective(s,[{questionId:"r1",value:"12 October"}]).score).toBe(1);
});
