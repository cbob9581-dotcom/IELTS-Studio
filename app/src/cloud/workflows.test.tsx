import {describe,it,expect,vi} from "vitest";
import {render,screen,fireEvent,waitFor} from "@testing-library/react";
import {CloudLibrary} from "./CloudLibrary";
import {AssessmentPanel} from "./AssessmentPanel";
import {DraftMedia} from "./DraftMedia";
import type {DesktopPort} from "../platform/desktop";
describe("cloud workflow controls",()=>{
 it("does not download every recording while its preview is closed",async()=>{
  const asset=vi.fn().mockResolvedValue(new ArrayBuffer(8));
  render(<DraftMedia job={{id:"j",kind:"generation",state:"review_required",stage:1,request:{},assets:[{id:"a",path:"a.wav",mediaType:"audio/wav"}]}} port={{cloudAsset:asset} as unknown as DesktopPort}/>);
  await new Promise(r=>setTimeout(r,20));expect(asset).not.toHaveBeenCalled();
 });
 it("shows the current operation instead of a meaningless stage number",async()=>{
  const job={id:"j",kind:"generation",state:"pending",stage:1,request:{section:"listening"},operation:{phase:"audio",label:"制作听力录音",done:12,total:65,unit:"条",startedAt:Date.now()}};
  const port={cloudList:async()=>[job],cloudRead:async()=>job} as unknown as DesktopPort;
  render(<CloudLibrary port={port} mode="training" section="listening" part="" kind="" onPublished={()=>{}}/>);
  fireEvent.click(await screen.findByRole("button",{name:/AI 出题.*听力/}));
  expect(await screen.findByText(/12\s*\/\s*65/)).toBeInTheDocument();
  expect(screen.queryByText(/已保存步骤/)).not.toBeInTheDocument();
 });
 it("generates only the selected training part and kind",async()=>{
  const create=vi.fn().mockResolvedValue({id:"job-1",kind:"generation",state:"review_required",stage:1,request:{},paper:{},steps:[]});
  const port={cloudList:async()=>[],cloudCreate:create} as unknown as DesktopPort;
  render(<CloudLibrary port={port} mode="training" section="reading" part="2" kind="gap_fill" onPublished={()=>{}}/>);
  fireEvent.change(screen.getByLabelText("出题主题（可选）"),{target:{value:"urban planning"}});
  fireEvent.click(screen.getByRole("button",{name:"AI 生成此项"}));
  await waitFor(()=>expect(create).toHaveBeenCalledWith("generation",{section:"reading",part:2,kind:"gap_fill",topic:"urban planning"},null));
 });
 it("reports failures without silently retrying paid steps",async()=>{
  const step=vi.fn().mockResolvedValue({id:"j",kind:"assessment",state:"failed",error:"额度不足",evaluations:[],request:{attemptId:"a"}});
  const port={cloudList:async()=>[],cloudCreate:async()=>({id:"j",kind:"assessment",state:"pending",request:{attemptId:"a"}}),cloudStep:step} as unknown as DesktopPort;
  render(<AssessmentPanel port={port} attemptId="a"/>);
  fireEvent.click(screen.getByRole("button",{name:"开始云端评测"}));
  await screen.findByText("额度不足");expect(step).toHaveBeenCalledTimes(1);
 });
});
