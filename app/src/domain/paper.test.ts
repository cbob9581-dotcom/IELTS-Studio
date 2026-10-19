import { test, expect } from "vitest";
import { validatePaper } from "./paper";
import { fixturePaper } from "../../test/fixtures/paper";
test('rejects invalid audio range and inconsistent playlist order',()=>{
 const p=fixturePaper();p.assets.push({id:'audio1',path:'audio.wav',sha256:'a'.repeat(64),mediaType:'audio/wav'});p.sections[0].audioAssetIds=['audio1'];
 p.sections[0].audioPlaylist=[{assetId:'audio1',part:null,origin:'uploaded',durationMs:10000,segments:[{part:1,startMs:3000,endMs:1000}]}];
 expect(validatePaper(p).some(i=>i.code==='audio_timeline')).toBe(true);
});
test("rejects duplicate numbering, unknown kinds, traversal and missing evidence", () => {
  const paper = fixturePaper();
  expect(validatePaper(paper)).toEqual([]);
  const bad = structuredClone(paper);
  bad.sections[0].questions.push({
    ...bad.sections[0].questions[0],
    id: "other",
  });
  expect(validatePaper(bad).some((x) => x.code === "duplicate_number")).toBe(
    true,
  );
  const malformed = JSON.parse(JSON.stringify(paper));
  malformed.sections[0].questions[0].kind = "mystery";
  expect(validatePaper(malformed).length).toBeGreaterThan(0);
  paper.assets.push({
    id: "bad",
    path: "../secret.wav",
    sha256: "a".repeat(64),
    mediaType: "audio/wav",
  });
  expect(validatePaper(paper).some((x) => x.code === "unsafe_path")).toBe(true);
  const missing = fixturePaper();
  missing.sections[0].questions[0].evidence = [];
  expect(
    validatePaper(missing).some((x) => x.code === "missing_evidence"),
  ).toBe(true);
});
