import {useState} from 'react';
import {it,expect,vi} from 'vitest';
import {render,screen,fireEvent,within} from '@testing-library/react';
import {ExamView} from './ExamView';
import {completeFixture} from '../../test/fixtures/complete-paper';
import {createAttempt} from '../domain/session';
import type {DesktopPort} from '../platform/desktop';

it('pairs each of three passages with its own questions and preserves answers and the same timer',()=>{
 const paper=completeFixture();const reading=paper.sections.find(s=>s.section==='reading')!;
 reading.paragraphs=[1,2,3].map(part=>({id:`p${part}`,part,text:`ARTICLE_${part}`}));
 const initial=createAttempt(paper,'training','reading');const changes=vi.fn();
 function Harness(){const[a,setA]=useState(initial);return <ExamView paper={paper} attempt={a} port={{} as DesktopPort} onChange={v=>{changes(v);setA(v);}} onSubmit={()=>{}} paused={false} onPause={()=>{}}/>;}
 render(<Harness/>);expect(screen.getByText('ARTICLE_1')).toBeInTheDocument();expect(screen.queryByText('ARTICLE_2')).not.toBeInTheDocument();
 expect(document.querySelectorAll('.question-block')).toHaveLength(13);
 fireEvent.change(screen.getByRole('textbox',{name:'第 1 题答案'}),{target:{value:'saved answer'}});
 fireEvent.click(screen.getByRole('button',{name:'查看解析'}));expect(screen.getAllByText('参考答案')).toHaveLength(13);
 fireEvent.click(screen.getByRole('button',{name:/第 2 篇/}));expect(screen.getByText('ARTICLE_2')).toBeInTheDocument();expect(document.querySelectorAll('.question-block')).toHaveLength(13);
 expect(screen.queryByText('参考答案')).not.toBeInTheDocument();expect(changes.mock.calls.at(-1)![0].revealedQuestions).toHaveLength(13);
 fireEvent.click(screen.getByRole('button',{name:/第 3 篇/}));expect(screen.getByText('ARTICLE_3')).toBeInTheDocument();expect(document.querySelectorAll('.question-block')).toHaveLength(14);
 fireEvent.click(within(screen.getByLabelText('题号导航')).getByRole('button',{name:/^1$/}));expect(screen.getByText('ARTICLE_1')).toBeInTheDocument();expect(screen.getByRole('textbox',{name:'第 1 题答案'})).toHaveValue('saved answer');
 expect(changes.mock.calls.at(-1)![0].clock).toEqual(initial.clock);
});
