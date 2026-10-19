import { test, expect } from "vitest";
import { SaveQueue } from "./desktop";
import type { AttemptSnapshot } from "../domain/session";
test("serial queue saves latest draft with returned revisions and preserves failures", async () => {
  const revisions: number[] = [];
  const values: string[] = [];
  const q = new SaveQueue(async (a, r) => {
    revisions.push(r);
    values.push(a.responses[0].value as string);
    if (a.responses[0].value === "FAIL") throw new Error("disk full");
    return { ...a, revision: r + 1 };
  }, 0);
  const a = {
    id: "a",
    revision: 0,
    responses: [{ questionId: "q", value: "first" }],
  } as AttemptSnapshot;
  q.enqueue(a);
  q.enqueue({ ...a, responses: [{ questionId: "q", value: "last" }] });
  await q.flush();
  expect(values.at(-1)).toBe("last");
  expect(revisions).toEqual([0]);
  q.enqueue({ ...a, responses: [{ questionId: "q", value: "FAIL" }] });
  await expect(q.flush()).rejects.toThrow("disk full");
  expect(q.dirty).toBe(true);
  q.enqueue({ ...a, responses: [{ questionId: "q", value: "retry" }] });
  await q.flush();
  expect(revisions.at(-1)).toBe(1);
  expect(q.dirty).toBe(false);
});
