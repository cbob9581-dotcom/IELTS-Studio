import { it, expect } from "vitest";
import { sourceRunner } from "./runner";
import type { DesktopPort } from "../platform/desktop";
it('prepares original PDF page images before a figure-bearing unit is extracted',async()=>{
 const events:string[]=[];const p:any={id:'p',documents:[{id:'d',extension:'pdf'}],units:[{id:'u',documentId:'d',start:0,end:0,state:'pending'}]};
 const port={sourceAction:async(action:string)=>action==='blocks'?[{id:'b1',page:28,imageAssetId:'crop'}]:p,prepareSourcePages:async()=>{events.push('original-page');},sourceStep:async()=>{events.push('extract');return {...p,units:[{...p.units[0],state:'complete'}]};}} as unknown as DesktopPort;
 await sourceRunner(port).run(p,['u']);expect(events).toEqual(['original-page','extract']);
});
it("persists work independently of subscribers and stops a failed unit without retrying it", async () => {
  let calls = 0;
  const p: any = {
    id: "p",
    documents: [],
    units: [{ id: "u", state: "pending" }],
  };
  const port = {
    sourceStep: async () => {
      calls++;
      return {
        ...p,
        units: [{ id: "u", state: "failed", error: "提取超时，原件已保留" }],
      };
    },
  } as unknown as DesktopPort;
  const runner = sourceRunner(port);
  const off = runner.subscribe(() => {});
  off();
  await runner.run(p, ["u"]);
  expect(calls).toBe(1);
  expect(runner.snapshot().busy).toBe(false);
  expect(runner.snapshot().project?.units[0].state).toBe("failed");
});
