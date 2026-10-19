import type { PaperVersion, Question, Section } from "../../src/domain/paper";
export function completeFixture(): PaperVersion {
  const counts = [40, 40, 2, 3];
  const sections: Section[] = ["listening", "reading", "writing", "speaking"];
  return {
    schemaVersion: 1,
    id: "structure-only",
    version: 1,
    title: "TEST ONLY complete structure",
    testType: "academic",
    status: "ready",
    provenance: {
      kind: "developer_fixture",
      source: "unit-test-only structural fixture, not a real exam",
    },
    assets: [
      {
        id: "audio",
        path: "media/test.wav",
        sha256: "0".repeat(64),
        mediaType: "audio/wav",
      },
      {
        id: "chart",
        path: "media/test.png",
        sha256: "0".repeat(64),
        mediaType: "image/png",
      },
    ],
    sections: sections.map((section, i) => ({
      section,
      title: section,
      durationMs: 60000,
      timingSource: "TEST ONLY rule fixture",
      paragraphs:
        section === "reading"
          ? [1, 2, 3].map((part) => ({
              id: `p${part}`,
              part,
              text: "test evidence",
            }))
          : [],
      audioAssetIds: section === "listening" ? ["audio"] : undefined,
      questions: Array.from({ length: counts[i] }, (_, j): Question => ({
        id: `${section}-${j + 1}`,
        number: j + 1,
        kind:
          section === "writing"
            ? "writing_task"
            : section === "speaking"
              ? "speaking_prompt"
              : "gap_fill",
        prompt: "TEST ONLY",
        part:
          section === "listening"
            ? Math.floor(j / 10) + 1
            : section === "reading"
              ? j < 13
                ? 1
                : j < 26
                  ? 2
                  : 3
              : j + 1,
        points: i < 2 ? 1 : 0,
        answer:
          i < 2 ? { accepted: ["test"], caseSensitive: false } : undefined,
        evidence: i < 2 ? ["test evidence"] : [],
        imageAssetId: section === "writing" && j === 0 ? "chart" : undefined,
      })),
    })),
  };
}
