import { test, expect } from "vitest";
import { shouldOpenPaper } from "./interaction";
test("E requires room focus and ignores editing or held key", () => {
  const input = { key: "e", repeat: false, focused: true, editing: false };
  expect(shouldOpenPaper(input)).toBe(true);
  expect(shouldOpenPaper({ ...input, focused: false })).toBe(false);
  expect(shouldOpenPaper({ ...input, editing: true })).toBe(false);
  expect(shouldOpenPaper({ ...input, repeat: true })).toBe(false);
});
