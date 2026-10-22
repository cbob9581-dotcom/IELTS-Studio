import {describe,it,expect,vi} from "vitest";
import {render,screen,fireEvent,waitFor} from "@testing-library/react";
import {CloudLibrary} from "./CloudLibrary";
import {JobProgress,failureAdvice} from "./JobProgress";
import type {DesktopPort} from "../platform/desktop";
import type {CloudJob} from "./types";
import {Library} from "../pages/Library";
import {fixturePaper} from "../../test/fixtures/paper";
const job:CloudJob={id:"j",kind:"generation",state:"review_required",stage:2,request:{section:"listening"},paper:{schemaVersion:1,id:"p",version:1,title:"A draft",status:"review_required",testType:"academic",provenance:{kind:"ai_generated",source:"local"},assets:[],sections:[]},validation:{pass:false,issues:[{questionId:"q1",reason:"Missing units"}]}};
describe("recoverable text-first cloud work",()=>{
 it("hides developer test papers until a user explicitly asks for test material",async()=>{
  const p=fixturePaper();const port={listPapers:async()=>[{...p,sections:["reading"]}],cloudList:async()=>[]} as unknown as DesktopPort;
  render(<Library port={port} mode="training" onStart={async()=>{}} onSettings={()=>{}}/>);
  const toggle=await screen.findByRole("checkbox",{name:"显示开发测试素材"});
  expect(screen.queryByText(p.title)).not.toBeInTheDocument();fireEvent.click(toggle);
  expect(await screen.findByText(p.title)).toBeInTheDocument();
 });
 it("does not offer publication of rejected content and provides a text-only repair",async()=>{
  const fix=vi.fn().mockResolvedValue({...job,state:"review_required"});
  const port={cloudList:async()=>[job],cloudRead:async()=>job,cloudFixText:fix} as unknown as DesktopPort;
  render(<CloudLibrary port={port} mode="training" section="listening" part="" kind="" onPublished={()=>{}}/>);
  fireEvent.click(await screen.findByRole("button",{name:/AI 出题.*听力/}));
  expect(screen.queryByRole("button",{name:"发布到本地题库"})).not.toBeInTheDocument();
  fireEvent.click(await screen.findByRole("button",{name:"AI 修复并复查（仅文字）"}));
  await waitFor(()=>expect(fix).toHaveBeenCalledWith("j"));
 });
 it("requires a separate explicit action before any listening synthesis",async()=>{
  const j={...job,audioPending:true,audioSegmentCount:31,validation:{pass:true,issues:[]}};
  const audio=vi.fn().mockResolvedValue(j);
  const port={cloudList:async()=>[j],cloudRead:async()=>j,cloudStartAudio:audio,cloudReview:async()=>j} as unknown as DesktopPort;
  render(<CloudLibrary port={port} mode="training" section="listening" part="" kind="" onPublished={()=>{}}/>);
  fireEvent.click(await screen.findByRole("button",{name:/AI 出题.*听力/}));
  expect(audio).not.toHaveBeenCalled();
  fireEvent.click(await screen.findByRole("button",{name:/单独合成听力.*31/}));
  await waitFor(()=>expect(audio).toHaveBeenCalledWith("j"));
 });
 it("offers cached import recovery and does not claim a stopped repair is running",async()=>{
  const j:CloudJob={...job,kind:"import",state:"failed",stage:3,request:{sourceName:"Book.pdf"},error:"答案来源缺失",operation:{phase:"repair",label:"正在反馈具体原因并修复",done:0,total:4,unit:"科",startedAt:Date.now()}};
  const recover=vi.fn().mockResolvedValue({id:"source-1"});
  const port={cloudList:async()=>[j],cloudRead:async()=>j,sourceAction:recover} as unknown as DesktopPort;
  render(<CloudLibrary port={port} mode="exam" section="listening" part="" kind="" onPublished={()=>{}}/>);
  fireEvent.click(await screen.findByRole("button",{name:/资料解析.*Book/}));
  fireEvent.click(await screen.findByRole("button",{name:"在资料库继续（复用已解析内容）"}));
  await waitFor(()=>expect(recover).toHaveBeenCalledWith("recover_import",{jobId:"j"}));
 });
 it("shows a stopped phase and explains an import-size failure as a source workflow issue",()=>{
  const j={...job,state:"failed",operation:{phase:"repair",label:"正在反馈原因并修复",done:0,total:1,unit:"部分",startedAt:Date.now()}};
  render(<JobProgress job={j} busy={false}/>);expect(screen.queryByText("正在反馈原因并修复")).not.toBeInTheDocument();
  expect(failureAdvice("解析结果超过单卷处理范围")).toContain("资料库");
 });
});
