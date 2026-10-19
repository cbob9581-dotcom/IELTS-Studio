import type { PaperVersion } from "../../src/domain/paper";
export function fixturePaper(): PaperVersion {
  return {
    schemaVersion: 1,
    id: "dev-reading",
    version: 1,
    title: "开发验证卷 · 城市能源",
    testType: "academic",
    status: "ready",
    provenance: {
      kind: "developer_fixture",
      source: "原创微型开发素材；非正式 IELTS 试卷",
    },
    assets: [],
    sections: [
      {
        section: "reading",
        title: "Urban energy",
        durationMs: 600000,
        timingSource: "开发训练时长 10 分钟，不是完整考试计时",
        paragraphs: [
          {
            id: "p1",
            part: 1,
            text: "A. The city installed solar panels on public buildings in 2022. These panels supply electricity during the day. Batteries store surplus power for evening use.",
          },
          {
            id: "p2",
            part: 1,
            text: "B. A recent study found that libraries used less electricity after replacing their lighting. Researchers warned that the cost of batteries still limited wider adoption.",
          },
        ],
        questions: [
          {
            id: "r1",
            number: 1,
            kind: "gap_fill",
            part: 1,
            points: 1,
            prompt:
              "Complete with ONE WORD: The city installed _____ panels on public buildings.",
            answer: { accepted: ["solar"], caseSensitive: false, maxWords: 1 },
            evidence: ["p1: solar panels"],
          },
          {
            id: "r2",
            number: 2,
            kind: "true_false_not_given",
            part: 1,
            points: 1,
            prompt: "Batteries store energy for use in the evening.",
            answer: { accepted: ["TRUE"], caseSensitive: false },
            evidence: ["p1: Batteries store surplus power for evening use."],
          },
          {
            id: "r3",
            number: 3,
            kind: "single_choice",
            part: 1,
            points: 1,
            prompt: "What limited wider adoption?",
            options: [
              "A. The cost of batteries",
              "B. A lack of daylight",
              "C. Library closures",
            ],
            answer: {
              accepted: ["A. The cost of batteries"],
              caseSensitive: false,
            },
            evidence: [
              "p2: the cost of batteries still limited wider adoption.",
            ],
          },
        ],
      },
    ],
  };
}
