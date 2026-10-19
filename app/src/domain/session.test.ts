import { test, expect } from "vitest";
import { fixturePaper } from "../../test/fixtures/paper";
import {completeFixture} from "../../test/fixtures/complete-paper";
import {
  beginSession,
  reduceSession,
  restoreClock,
  nextSection,
  selectScope,
  advanceClock,
  createNextExamAttempt,
  createAttempt,
} from "./session";
test("whole exam advances only after submission and keeps its frozen paper and run",()=>{
 const p=completeFixture();p.provenance.kind="imported";
 const a=createAttempt(p,"exam","listening");
 expect(()=>createNextExamAttempt(p,a)).toThrow();
 const b=createNextExamAttempt(p,{...a,status:"submitted"})!;
 expect(b.section).toBe("reading");expect(b.examRunId).toBe(a.examRunId);expect(b.paperVersion).toBe(a.paperVersion);expect(b.scope).toEqual({});
 expect(createNextExamAttempt(p,{...b,status:"submitted",section:"speaking"})).toBeNull();
});
test("exam rejects developer suite and training selects only requested section", () => {
  const p = fixturePaper();
  expect(() => beginSession(p, "exam")).toThrow();
  const s = beginSession(p, "training", "reading");
  expect(s.section).toBe("reading");
  expect(s.phase).toBe("room");
  expect(() => beginSession(p, "training", "writing")).toThrow();
});
test("submitting and submitted states cannot submit twice", () => {
  const s = beginSession(fixturePaper(), "training", "reading");
  const a = reduceSession(s, { type: "OPEN", now: 1000 });
  expect(a.phase).toBe("answering");
  const b = reduceSession(a, { type: "SUBMIT" });
  expect(b.phase).toBe("submitting");
  expect(reduceSession(b, { type: "SUBMIT" })).toEqual(b);
});
test("resume and clock rollback cannot create extra exam time", () => {
  const clock = {
    durationMs: 60000,
    deadlineUtcMs: 61000,
    lastObservedUtcMs: 11000,
    remainingMs: 50000,
  };
  expect(restoreClock(clock, 71000).remainingMs).toBe(0);
  expect(restoreClock(clock, 500).remainingMs).toBe(50000);
  expect(
    restoreClock({ ...clock, deadlineUtcMs: null }, 999999).remainingMs,
  ).toBe(50000);
  expect(nextSection("listening")).toBe("reading");
  expect(nextSection("writing")).toBe("speaking");
  expect(nextSection("speaking")).toBe(null);
});
test("scope excludes outside parts and does not mutate frozen paper", () => {
  const p = fixturePaper();
  const before = JSON.stringify(p);
  const section = selectScope(p.sections[0], { kind: "single_choice" });
  expect(section.questions.map((q) => q.id)).toEqual(["r3"]);
  expect(JSON.stringify(p)).toBe(before);
});
test("fractional performance ticks persist integer milliseconds accepted by Rust", () => {
  const clock = {
    durationMs: 60000,
    deadlineUtcMs: null,
    lastObservedUtcMs: 1000,
    remainingMs: 60000,
  };
  expect(advanceClock(clock, 2000, 1000.25).remainingMs).toBe(58999);
});
