import {test,expect} from 'vitest';
import {sourceHealth} from './health';
test('a book reports useful completed work and exact missing answers without claiming a full exam is ready',()=>{
 const p:any={units:[{id:'u1',test:'Test 1',role:'questions',state:'complete',confirmed:true,result:{section:{section:'reading',questions:[{id:'q1',number:1,kind:'gap_fill',answer:{accepted:['river']},answerSource:{quote:'1 river'}},{id:'q2',number:2,kind:'gap_fill'}]}}},{id:'u2',test:'Test 2',role:'questions',state:'failed',error:'missing image'}],documents:[]};
 const h=sourceHealth(p);expect(h[0].questions).toBe(2);expect(h[0].missingAnswers).toEqual(['reading 2']);expect(h[1].failures[0].id).toBe('u2');expect(h[0].status).toBe('needs_answers');
});
