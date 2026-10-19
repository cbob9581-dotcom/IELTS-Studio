import {it,expect} from 'vitest';
import {readFileSync} from 'node:fs';
import {render,cleanup} from '@testing-library/react';
import {gradeObjective} from './grading';
import {QuestionRenderer} from '../exam/QuestionRenderer';
import type {PaperVersion} from './paper';
it.skipIf(!process.env.IELTS_REAL_PAPER)('real Cambridge 17 has 40 score positions, usable widgets, and correct partial credit',()=>{
 const p=JSON.parse(readFileSync(process.env.IELTS_REAL_PAPER!,'utf8')) as PaperVersion;const s=p.sections.find(s=>s.section==='reading')!;
 const result=gradeObjective(s,s.questions.map(q=>({questionId:q.id,value:q.answer!.accepted[0]})));expect(result.score).toBe(40);expect(result.total).toBe(40);
 const q16=s.questions.find(q=>q.number===16)!;expect(Object.values(q16.answer!.accepted[0])).toContain('E');
 for(const q of s.questions.filter(q=>q.kind==='matching')){const ui=render(<QuestionRenderer question={q} value="" onChange={()=>{}}/>);expect(ui.container.querySelectorAll('select').length).toBeGreaterThan(0);cleanup();}
 const q23=s.questions.find(q=>q.number===23)!;expect(q23.points).toBe(2);expect(s.questions.some(q=>q.number===24)).toBe(false);
 const pair=q23.answer!.accepted[0] as string[];const partial=gradeObjective({...s,questions:[q23]},[{questionId:q23.id,value:[pair[0]]}]);expect(partial.score).toBe(1);expect(partial.total).toBe(2);
});
