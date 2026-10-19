import {isQuestionFigure} from "./pageFigures";
import type {Block} from "./types";
import type { DesktopPort } from "../platform/desktop";
import { errorText } from "../platform/desktop";
import type { SourceProject } from "./types";
type Snapshot = {
  busy: boolean;
  project: SourceProject | null;
  targetId: string;
  error: string;
  paused: boolean;
};
const runners = new WeakMap<DesktopPort, Runner>();
class Runner {
  private state: Snapshot = {
    busy: false,
    project: null,
    targetId: "",
    error: "",
    paused: false,
  };
  private listeners = new Set<() => void>();
  private stop = false;
  constructor(private port: DesktopPort) {}
  snapshot = () => this.state;
  subscribe = (f: () => void) => {
    this.listeners.add(f);
    return () => {
      this.listeners.delete(f);
    };
  };
  private emit(v: Partial<Snapshot>) {
    this.state = { ...this.state, ...v };
    this.listeners.forEach((f) => f());
  }
  pause = () => {
    this.stop = true;
    this.emit({ paused: true });
  };
  async run(p: SourceProject, targets: string[]) {
    if (this.state.busy) throw Error("已有资料任务运行，请先暂停或等待完成");
    if (!this.port.sourceStep) throw Error("请使用支持资料库的新版桌面程序");
    this.stop = false;
    this.emit({ busy: true, project: p, error: "", paused: false });
    let version = 0,
      polling = false;
    const timer = setInterval(async () => {
      if (polling || !this.port.sourceAction) return;
      polling = true;
      const v = version;
      try {
        const current = await this.port.sourceAction("read", { id: p.id });
        if (v === version && this.state.busy) this.emit({ project: current });
      } catch {
        /* Step response owns actionable errors. */
      } finally {
        polling = false;
      }
    }, 1500);
    try {
      for (const id of targets) {
        if (this.stop) break;
        this.emit({ targetId: id });
        const unit=p.units.find(u=>u.id===id),doc=unit?p.documents.find(d=>d.id===unit.documentId):undefined;
        if(unit&&doc?.extension==="pdf"&&this.port.prepareSourcePages&&this.port.sourceAction){
          const blocks:Block[]=await this.port.sourceAction("blocks",{id:p.id,targetId:doc.id});
          const pages=blocks.slice(unit.start,unit.end+1).filter(b=>unit.role==='answers'||isQuestionFigure(b)).map(b=>b.page).filter((n):n is number=>!!n);
          if(pages.length){await this.port.prepareSourcePages(p.id,doc.id,[...new Set(pages)]);p=await this.port.sourceAction("read",{id:p.id});version++;this.emit({project:p});}
        }
        while (!this.stop) {
          p = await this.port.sourceStep(p.id, id);
          version++;
          this.emit({ project: p });
          const target =
            p.documents.find((d) => d.id === id) ??
            p.units.find((u) => u.id === id) ?? p.audioDrafts?.find(d=>d.id===id);
          if (
            !target ||
            ["complete", "ready", "media", "failed"].includes(target.state)
          )
            break;
          await new Promise((r) =>
            setTimeout(r, target.state === "parsing" ? 3000 : 100),
          );
        }
        if (
          (
            p.documents.find((d) => d.id === id) ??
            p.units.find((u) => u.id === id) ?? p.audioDrafts?.find(d=>d.id===id)
          )?.state === "failed"
        )
          break;
      }
    } catch (e) {
      this.emit({ error: errorText(e) });
      throw e;
    } finally {
      this.port.releaseSourcePages?.();
      clearInterval(timer);
      this.emit({ busy: false });
    }
    return p;
  }
}
export function sourceRunner(p: DesktopPort) {
  let r = runners.get(p);
  if (!r) {
    r = new Runner(p);
    runners.set(p, r);
  }
  return r;
}
