// Acceptance-only fixture; not a production entry point or desktop fallback.
import {useState} from 'react';
import {createRoot} from 'react-dom/client';
import {ExamView} from '../../src/exam/ExamView';
import {createAttempt,type AttemptSnapshot} from '../../src/domain/session';
import type {PaperVersion,Question} from '../../src/domain/paper';
import {browserTestPort} from '../../src/platform/browser-test';
import reading from '../fixtures/dev-reading.json';
import writing from '../fixtures/dev-writing.json';
import '../../src/styles.css';
const paper=structuredClone(reading) as PaperVersion;
paper.title='Long article visual acceptance fixture';paper.assets=writing.assets;
paper.sections[0].paragraphs=Array.from({length:14},(_,i)=>({id:`long-${i}`,part:1,text:`Paragraph ${i+1}. `+'Libraries provide spaces where people can study independently. Researchers compared energy use and visitor numbers across different public buildings. The findings suggest that natural daylight and quiet spaces contribute to effective study. '.repeat(4)+(i===13?'END OF LONG ARTICLE.':'')}));
paper.sections[0].questions.push({id:'diagram',number:2,kind:'diagram_label',part:1,points:1,prompt:'Label the original developer chart.',imageAssetId:'visitors-chart',labels:[{id:'A',x:.5,y:.3}],answer:{accepted:[{A:'Wednesday'}],caseSensitive:false},evidence:['developer chart']} satisfies Question);
const port={...browserTestPort,readAsset:async()=>Array.from(new Uint8Array(await (await fetch('/test/fixtures/media/library-visitors.png')).arrayBuffer()))};
function Fixture(){const [attempt,setAttempt]=useState<AttemptSnapshot>(()=>({...createAttempt(paper,'training','reading'),status:'answering',mode:new URLSearchParams(location.search).get('mode')==='exam'?'exam':'training'}));return <div style={{padding:12}}><ExamView paper={paper} attempt={attempt} port={port} onChange={setAttempt} onSubmit={()=>{}} paused={false} onPause={()=>{}}/></div>;}
createRoot(document.getElementById('root')!).render(<Fixture/>);
