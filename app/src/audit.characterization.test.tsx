// Regression acceptance tests converted from the 0.4.1 audit probes.
import {test,expect,vi,afterEach} from 'vitest';
import {render,fireEvent,cleanup} from '@testing-library/react';
import {ListeningPlayer} from './exam/ListeningPlayer';
import {selectScope, type AttemptSnapshot} from './domain/session';
import {completeFixture} from '../test/fixtures/complete-paper';
import {createSpeakingState,reduceSpeaking} from './speaking/flow';
import {SaveQueue} from './platform/desktop';
afterEach(()=>{cleanup();vi.restoreAllMocks();});
test('listening automatically continues and exam completion cannot replay',async()=>{
 const play=vi.spyOn(HTMLMediaElement.prototype,'play').mockResolvedValue();
 const v=render(<ListeningPlayer urls={['blob:first','blob:second']} mode="exam" paused={false} onReady={()=>{}} onError={()=>{}}/>);
 fireEvent.click(v.getByRole('button',{name:'播放音频 1/2'}));
 await vi.waitFor(()=>expect(play).toHaveBeenCalledTimes(1));
 fireEvent.ended(v.container.querySelector('audio')!);
 expect(v.container.querySelector('audio')!.getAttribute('src')).toBe('blob:second');
 await vi.waitFor(()=>expect(play).toHaveBeenCalledTimes(2));
 fireEvent.ended(v.container.querySelector('audio')!);
 expect(v.getByRole('button',{name:'听力播放已完成'})).toBeDisabled();
});
test('Part 2 selection includes only explicitly mapped audio',()=>{
 const s=completeFixture().sections[0];s.audioAssetIds=['part1','part2','part3','part4'];
 Object.assign(s,{audioParts:{part1:1,part2:2,part3:3,part4:4}});
 const selected=selectScope(s,{part:2});
 expect(selected.questions.every(q=>q.part===2)).toBe(true);
 expect(selected.audioAssetIds).toEqual(['part2']);
});
test('provider wait does not exhaust Part 1 after one short answer',()=>{
 const qs=completeFixture().sections[3].questions;
 let s=createSpeakingState(qs,{part:1},0);
 s=reduceSpeaking(s,{type:'audio_ended',turnId:s.turnId,now:300000});
 s=reduceSpeaking(s,{type:'answer',turnId:s.turnId,text:'Ten second answer',recordingId:'r1',now:310000});
 expect(s.turns).toHaveLength(1);expect(s.stage).toBe('question');
});
test('save-only retry clears dirty flag after recovery',async()=>{
 let first=true;
 const q=new SaveQueue(async(a,r)=>{if(first){first=false;throw Error('temporary failure');}return {...a,revision:r+1};},0);
 q.enqueue({id:'audit',revision:0,responses:[]} as unknown as AttemptSnapshot);
 await expect(q.flush()).rejects.toThrow('temporary failure');
 await q.flush();expect(q.revision).toBe(1);expect(q.dirty).toBe(false);
});
