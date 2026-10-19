import { test, expect } from "vitest";
import { fixturePaper } from "../../test/fixtures/paper";
import { checkReadiness } from "./readiness";
import { completeFixture } from "../../test/fixtures/complete-paper";
test("developer fixtures never start exams but selected section may train", () => {
  const p = fixturePaper();
  expect(
    checkReadiness(p, "exam").some((x) => x.code === "developer_fixture"),
  ).toBe(true);
  expect(checkReadiness(p, "training", "reading")).toEqual([]);
  expect(checkReadiness(p, "training", "listening").length).toBeGreaterThan(0);
});
test("missing listening audio blocks readiness", () => {
  const p = fixturePaper();
  p.sections[0].section = "listening";
  expect(
    checkReadiness(p, "training", "listening").some(
      (x) => x.code === "missing_audio",
    ),
  ).toBe(true);
});
test("complete Academic test needs exact part numbers and sequential question numbers", () => {
  const paper = completeFixture();
  expect(checkReadiness(paper, "exam").map((i) => i.code)).toEqual([
    "developer_fixture",
  ]);
  paper.provenance.kind = "imported";
  expect(checkReadiness(paper, "exam")).toEqual([]);
  paper.sections[2].questions[1].part = 3;
  expect(
    checkReadiness(paper, "exam").some(
      (i) => i.code === "incomplete_structure",
    ),
  ).toBe(true);
});
test("two numbered answers in a multiple-choice group count as two of forty marks",()=>{
 const p=completeFixture();p.provenance.kind="imported";
 const s=p.sections[0],first=s.questions[0];s.questions.splice(0,2,{...first,kind:"multiple_choice",points:2,options:["A","B","C"],answer:{accepted:[["A","B"]],caseSensitive:false}});
 expect(checkReadiness(p,"exam")).toEqual([]);
});
