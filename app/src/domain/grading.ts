import type {
  SectionPaper,
  Response,
  ResponseValue,
  AnswerRule,
} from "./paper";
export type ObjectiveResult = {
  score: number;
  total: number;
  items: {
    questionId: string;
    correct: boolean | null;
    score: number;
    max: number;
  }[];
};
function normalize(v: ResponseValue, r: AnswerRule): string {
  const text = (x: string) => {
    const s = x.trim().replace(/\s+/g, " ");
    return r.caseSensitive ? s : s.toLocaleLowerCase("en");
  };
  if (typeof v === "string") return JSON.stringify(text(v));
  if (Array.isArray(v)) return JSON.stringify([...new Set(v.map(text))].sort());
  return JSON.stringify(
    Object.entries(v)
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([k, x]) => [k, text(x)]),
  );
}
export function gradeObjective(
  p: SectionPaper,
  responses: Response[],
  revealedQuestions: string[] = [],
): ObjectiveResult {
  const items = p.questions.map((q) => {
    if (revealedQuestions.includes(q.id)) return {questionId:q.id,correct:null,score:0,max:0};
    if (!q.answer) return { questionId: q.id, correct: null, score: 0, max: 0 };
    const v = responses.find((x) => x.questionId === q.id)?.value ?? "";
    const r = q.answer;
    const values =
      typeof v === "string" ? [v] : Array.isArray(v) ? v : Object.values(v);
    const valid = values.every(s=>{
      const tokens=s.trim().split(/\s+/).filter(Boolean);
      const isNumber=(token:string)=>/^\p{Sc}?[+-]?\d+(?:[.,:/-]\d+)*(?:st|nd|rd|th)?%?$/iu.test(token);
      const numbers=r.maxWords===0&&r.maxNumbers===1&&tokens.length>0&&tokens.every(isNumber)?1:tokens.filter(isNumber).length;
      const words=r.maxNumbers===undefined?tokens.length:tokens.filter(t=>!isNumber(t)).length;
      return (r.maxWords===undefined||words<=r.maxWords)&&(r.maxNumbers===undefined||numbers<=r.maxNumbers);
    });
    const correct =
      valid && r.accepted.some((a) => normalize(a, r) === normalize(v, r));
    let score=correct?q.points:0;
    if(q.kind==="multiple_choice"&&Number.isInteger(q.points)&&q.points>1&&Array.isArray(v)){
      const normalizeOption=(s:string)=>r.caseSensitive?s.trim():s.trim().toLocaleLowerCase("en");
      const selected=[...new Set(v.map(normalizeOption))];
      score=valid&&selected.length<=q.points?Math.max(0,...r.accepted.filter(Array.isArray).filter(a=>a.length===q.points).map(a=>a.map(normalizeOption).filter(x=>selected.includes(x)).length)):0;
    }
    return {
      questionId: q.id,
      correct:score===q.points,
      score,
      max: q.points,
    };
  });
  return {
    score: items.reduce((n, x) => n + x.score, 0),
    total: items.reduce((n, x) => n + x.max, 0),
    items,
  };
}
