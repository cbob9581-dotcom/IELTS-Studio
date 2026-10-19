import {render,screen,waitFor} from '@testing-library/react';
import {it,expect,vi} from 'vitest';
import {ExamView} from './ExamView';
import {completeFixture} from '../../test/fixtures/complete-paper';
import {createAttempt} from '../domain/session';
import type {DesktopPort} from '../platform/desktop';
it.each(['exam','training'] as const)('transcript visibility and binary media in %s mode',async mode=>{
 const paper=completeFixture();paper.sections[0].audioScripts=[{part:1,origin:'book',turns:[{id:'t1',speaker:'PETER',text:'SAMPLE_TRANSCRIPT_SECRET'}]}];
 const attempt={...createAttempt(paper,'training','listening'),mode};
 const binary=vi.fn().mockResolvedValue(new ArrayBuffer(44));
 const create=URL.createObjectURL,revoke=URL.revokeObjectURL;URL.createObjectURL=()=> 'blob:audio';URL.revokeObjectURL=()=>{};
 try{
  render(<ExamView paper={paper} attempt={attempt} port={{readAssetBinary:binary} as unknown as DesktopPort} onChange={()=>{}} onSubmit={()=>{}} paused={false} onPause={()=>{}}/>);
  await waitFor(()=>expect(binary).toHaveBeenCalled());
  if(mode==='exam')expect(screen.queryByText('SAMPLE_TRANSCRIPT_SECRET')).toBeNull();else expect(screen.getByText('SAMPLE_TRANSCRIPT_SECRET')).toBeInTheDocument();
 }finally{URL.createObjectURL=create;URL.revokeObjectURL=revoke;}
});
