import {test,expect,vi} from "vitest";
import {taskRunner} from "./taskRunner";
import type {DesktopPort} from "../platform/desktop";
import type {CloudJob} from "./types";
test("cloud task continues with no page subscriber and does not repeat completed steps",async()=>{
 let release!:(v:CloudJob)=>void;
 const initial={id:"task",kind:"generation",request:{},state:"pending",stage:0} as CloudJob;
 const port={cloudStep:vi.fn().mockImplementationOnce(()=>new Promise(r=>release=r)).mockResolvedValue({...initial,stage:2,state:"review_required"})} as unknown as DesktopPort;
 const runner=taskRunner(port);const unsubscribe=runner.subscribe(()=>{});
 const result=runner.run(initial);unsubscribe();
 release({...initial,stage:1});
 expect((await result).state).toBe("review_required");
 expect(port.cloudStep).toHaveBeenCalledTimes(2);expect(runner.snapshot().busy).toBe(false);
});
test("failed cloud step stops without automatic paid retry",async()=>{
 const port={cloudStep:vi.fn().mockResolvedValue({id:"x",kind:"import",request:{},state:"failed",stage:0,error:"timeout"})} as unknown as DesktopPort;
 const runner=taskRunner(port);
 await runner.run({id:"x",kind:"import",request:{},state:"pending",stage:0});
 expect(port.cloudStep).toHaveBeenCalledTimes(1);expect(runner.snapshot().job?.state).toBe("failed");
});
