import {render,screen,fireEvent,waitFor} from '@testing-library/react';
import {it,expect,vi} from 'vitest';
import {ListeningAudioPanel} from './ListeningAudioPanel';
import type {DesktopPort} from '../platform/desktop';
import type {SourceProject} from './types';
const p={id:'project',revision:1,title:'Book',units:[],documents:[],entries:[],publications:[]} as SourceProject;
function mount(project=p){const action=vi.fn().mockResolvedValue(project);const port={getConfig:vi.fn().mockResolvedValue({providers:{qwenTts:{voice:'Aiden|Cherry'}}}),chooseFiles:vi.fn().mockResolvedValue(['a.mp3','b.mp3'])} as unknown as DesktopPort;render(<ListeningAudioPanel port={port} project={project} busy={false} action={action} run={vi.fn()} report={vi.fn()}/>);return {action,port};}
it('user text creates a draft without OCR or book extraction',async()=>{const {action}=mount();fireEvent.click(screen.getByRole('button',{name:'用户文本合成'}));fireEvent.change(screen.getByLabelText('用户听力文本'),{target:{value:'JAN: Hello.'}});fireEvent.click(screen.getByRole('button',{name:'从文本创建听力稿'}));await waitFor(()=>expect(action).toHaveBeenCalledWith('transcript_import',expect.objectContaining({text:'JAN: Hello.'})));expect(action).toHaveBeenCalledTimes(1);});
it('multiple audio files go through upload without generation',async()=>{const {action}=mount();fireEvent.click(screen.getByRole('button',{name:'上传音频'}));fireEvent.click(screen.getByRole('button',{name:'选择一个或多个音频文件'}));await waitFor(()=>expect(action).toHaveBeenCalledWith('add_documents',{files:[{path:'a.mp3',roles:['audio']},{path:'b.mp3',roles:['audio']}]}));expect(action).toHaveBeenCalledTimes(1);});
it('book scripts use their own extraction entry',async()=>{const {action}=mount({...p,units:[{id:'u1',test:'Test 1',part:1,role:'transcript'}] as SourceProject['units']});fireEvent.click(screen.getByRole('button',{name:'Test 1 Part 1'}));await waitFor(()=>expect(action).toHaveBeenCalledWith('transcript_extract',{targetId:'u1'}));});
