import {it,expect,vi} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@testing-library/react';
import {CloudLibrary} from './CloudLibrary';
import {taskRunner} from './taskRunner';
import type {DesktopPort} from '../platform/desktop';
import type {CloudJob} from './types';
it('finishes approved generation without making the learner inspect or confirm answers',async()=>{
 const job={id:'student',kind:'generation',state:'review_required',stage:3,request:{section:'reading'},paper:{sections:[]},validation:{pass:true,issues:[]},audioPending:false} as unknown as CloudJob;
 const publish=vi.fn().mockResolvedValue({...job,state:'complete'});const port={cloudPublish:publish} as unknown as DesktopPort;
 expect((await taskRunner(port).run(job)).state).toBe('complete');expect(publish).toHaveBeenCalledTimes(1);
});
it('does not automatically publish a failed audit or synthesize missing audio',async()=>{
 const job={id:'blocked',kind:'generation',state:'review_required',stage:3,request:{},validation:{pass:false,issues:[]}} as CloudJob;const publish=vi.fn(),tts=vi.fn();const port={cloudPublish:publish,cloudStartAudio:tts} as unknown as DesktopPort;
 await taskRunner(port).run(job);expect(publish).not.toHaveBeenCalled();expect(tts).not.toHaveBeenCalled();
});
it('keeps authoring tools optional rather than opening an answer-editing worksheet for learners',async()=>{
 const job={id:'pending',kind:'generation',state:'review_required',stage:3,request:{section:'reading'},paper:{schemaVersion:1,id:'p',version:1,sections:[],assets:[]},validation:{pass:false,issues:[]}} as unknown as CloudJob;
 const port={cloudList:async()=>[job],cloudRead:async()=>job} as unknown as DesktopPort;
 render(<CloudLibrary port={port} mode="training" section="reading" part="" kind="" onPublished={()=>{}}/>);fireEvent.click(await screen.findByRole('button',{name:/AI 出题.*阅读/}));
 await waitFor(()=>expect(screen.queryByRole('checkbox',{name:/已核对原文/})).not.toBeInTheDocument());expect(screen.getByText('资料管理与编辑（可选）')).toBeInTheDocument();expect(screen.queryByRole('button',{name:'重新校审当前草稿'})).not.toBeInTheDocument();
});
