import type {ScriptDraft} from './types';
export function splitScriptTurn(d:ScriptDraft,id:string):ScriptDraft{
 const index=d.turns.findIndex(t=>t.id===id);if(index<0)return d;const t=d.turns[index],half=t.text.length/2;
 const boundaries=[...t.text.matchAll(/[.!?]\s+/g)].map(m=>m.index!+1);const spaces=[...t.text.matchAll(/\s+/g)].map(m=>m.index!);
 const cuts=(boundaries.length?boundaries:spaces).filter(n=>n>0&&n<t.text.length-1);if(!cuts.length)return d;
 const cut=cuts.reduce((a,b)=>Math.abs(b-half)<Math.abs(a-half)?b:a);const next={...t,id:`t-${crypto.randomUUID()}`,text:t.text.slice(cut).trimStart()};
 return {...d,turns:[...d.turns.slice(0,index),{...t,text:t.text.slice(0,cut).trimEnd()},next,...d.turns.slice(index+1)],events:d.events.flatMap(e=>e.type==='speech'&&e.turnId===id?[e,{type:'speech' as const,turnId:next.id}]:[e])};
}
