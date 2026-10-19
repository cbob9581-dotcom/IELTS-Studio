import {describe,it,expect} from 'vitest';
import {selectScope,createAttempt} from '../domain/session';
import {completeFixture} from '../../test/fixtures/complete-paper';
import type {SectionPaper} from '../domain/paper';
import {splitScriptTurn} from './listeningAudio';
import type {ScriptDraft} from './types';
it('splitting preserves words, voice, original source and existing pauses',()=>{
 const text='The island’s a great place to explore. It is quite small. It has several habitats.';
 const draft:ScriptDraft={id:'draft',revision:0,origin:'user',test:'Test 1',part:1,state:'draft',sourceRefs:[],rawText:'original',turns:[{id:'t1',speaker:'PETER',text}],events:[{type:'speech',turnId:'t1'},{type:'silence',durationMs:600}],voices:{PETER:'Aiden'}};
 const result=splitScriptTurn(draft,'t1');expect(result.turns.map(t=>t.text).join(' ')).toBe(text);expect(result.turns.every(t=>t.speaker==='PETER')).toBe(true);expect(result.rawText).toBe('original');expect(result.events.map(e=>e.type)).toEqual(['speech','speech','silence']);
});
describe('whole listening audio',()=>{
 const section={section:'listening',questions:[{part:1},{part:2}],paragraphs:[],audioAssetIds:['whole'],audioPlaylist:[{assetId:'whole',part:null,origin:'uploaded',segments:[]}]} as unknown as SectionPaper;
 it('requires manual markers for part training',()=>{expect(()=>selectScope(section,{part:2})).toThrow(/标记/);});
 it('uses marked range instead of replaying whole test',()=>{
  const s=structuredClone(section);s.audioPlaylist![0].segments=[{part:2,startMs:1000,endMs:4000}];
  const selected=selectScope(s,{part:2});expect(selected.audioPlaylist![0].startMs).toBe(1000);expect(selected.audioPlaylist![0].endMs).toBe(4000);expect(selected.audioAssetIds).toEqual(['whole']);
 });
});
it('part practice clock follows the selected marked range',()=>{
 const p=completeFixture();p.sections[0].audioPlaylist=[{assetId:'audio',part:null,origin:'uploaded',durationMs:10000,segments:[{part:2,startMs:1000,endMs:4000}]}];
 const a=createAttempt(p,'training','listening',{part:2});expect(a.clock.durationMs).toBe(3000);expect(a.clock.remainingMs).toBe(3000);
});
