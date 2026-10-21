import type {SourceProject} from './types';
export function sourceHealth(p:SourceProject){return [...new Set(p.units.map(u=>u.test))].sort().map(test=>{
 const units=p.units.filter(u=>u.test===test),questions=units.filter(u=>u.role==='questions').flatMap(u=>(u.result?.section?.questions??[]).map(q=>({q,section:u.section??u.result?.section?.section??'unknown'}))),missingAnswers=questions.filter(({q})=>!['writing_task','speaking_prompt'].includes(q.kind)&&(!q.answer?.accepted?.length||!q.answerSource?.quote)).map(({q,section})=>`${section} ${q.number}`);
 const failures=units.filter(u=>u.state==='failed').map(u=>({id:u.id,title:u.title,error:u.error}));
 const sections=new Set(units.filter(u=>u.role==='questions'&&u.state==='complete').map(u=>u.section));
 return {test,questions:questions.length,missingAnswers,failures,pending:units.filter(u=>!['complete','failed'].includes(u.state)).length,status:missingAnswers.length?'needs_answers':questions.length?'extracted':'materials_only',sections:[...sections]};
 });}
