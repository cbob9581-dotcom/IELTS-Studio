import {render,screen,fireEvent,waitFor} from '@testing-library/react';
import {it,expect,vi} from 'vitest';
import {TaskCenter} from './TaskCenter';
import type {DesktopPort} from '../platform/desktop';
it('opens the exact source and resumes only an interrupted native task',async()=>{
 const task={id:'queue-one',kind:'source',owner:'book-one',targets:['first','second'],completed:['first'],title:'Test 1 阅读',state:'paused',createdAt:1,updatedAt:2,error:null,usage:{requests:3,inputTokens:20,outputTokens:null}};
 const resume=vi.fn().mockResolvedValue(task),open=vi.fn();const port={taskList:async()=>[task],taskResume:resume} as unknown as DesktopPort;
 render(<TaskCenter port={port} onOpen={open}/>);await screen.findByRole('heading',{name:'Test 1 阅读'});expect(screen.getByText(/1\/2 个处理单元已完成/)).toBeInTheDocument();expect(screen.getByText(/输入 20.*输出 未提供/)).toBeInTheDocument();fireEvent.click(screen.getByRole('button',{name:'查看对应内容'}));expect(open).toHaveBeenCalledWith(task);fireEvent.click(screen.getByRole('button',{name:'继续未完成步骤'}));await waitFor(()=>expect(resume).toHaveBeenCalledWith('queue-one'));
});
it('keeps an explicit native policy-save error visible',async()=>{
 const port={taskList:async()=>[],taskPolicyRead:async()=>({backgroundConcurrency:2,ocrConcurrency:2,ttsConcurrency:2,maxTextRequests:40}),taskPolicySave:async()=>{throw Error('请先暂停正在执行的任务');}} as unknown as DesktopPort;
 render(<TaskCenter port={port} onOpen={()=>{}}/>);fireEvent.click(await screen.findByText('任务并发与请求预算'));fireEvent.click(screen.getByRole('button',{name:'保存任务策略'}));expect(await screen.findByRole('alert')).toHaveTextContent('请先暂停正在执行的任务');
});
