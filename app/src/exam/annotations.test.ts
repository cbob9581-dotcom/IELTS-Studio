import { test, expect } from "vitest";
import { selectedAnnotation } from "./annotations";
test("selection preserves text offsets even when existing highlights split nodes", () => {
  const p = document.createElement("p");
  p.dataset.paragraph = "p1";
  p.innerHTML = "Solar <mark>panels</mark> supply power.";
  const r = document.createRange();
  r.setStart(p.firstChild!, 0);
  r.setEnd(p.querySelector("mark")!.firstChild!, 6);
  const a = selectedAnnotation(p, r, "evidence");
  expect(a?.start).toBe(0);
  expect(a?.end).toBe(12);
  expect(a?.text).toBe("Solar panels");
  const outside = document.createElement("p");
  outside.textContent = "other";
  const bad = document.createRange();
  bad.selectNodeContents(outside);
  expect(selectedAnnotation(p, bad, "")).toBeNull();
});
