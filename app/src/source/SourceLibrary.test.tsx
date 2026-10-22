import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { SourceLibrary } from "./SourceLibrary";
import type { DesktopPort } from "../platform/desktop";
describe("source workspace", () => {
  it("uses completed test material without asking learners to certify the answer key",async()=>{
    const project={id:"p",title:"Book",revision:1,documents:[],units:[{id:"u",test:"Test 1",section:"reading",part:1,role:"questions",state:"complete",confirmed:true,result:{section:{questions:[{id:"q",number:1,answer:{accepted:["answer"]},sourceAnchors:[]}]}}}],entries:[],publications:[]};
    const action=vi.fn(async(a:string)=>a==="list"?[project]:project);
    render(<SourceLibrary port={{sourceAction:action} as unknown as DesktopPort} initialProjectId="p"/>);
    const usePaper=await screen.findByRole("button",{name:"使用这套试卷"});expect(usePaper).toBeEnabled();expect(screen.queryByRole("checkbox",{name:/已对照原件核对/})).not.toBeInTheDocument();fireEvent.click(usePaper);
    await waitFor(()=>expect(action).toHaveBeenCalledWith("publish",expect.objectContaining({test:"Test 1",units:["u"]})));
  });
  it("creates a named Academic project and offers supplementary answer/explanation uploads", async () => {
    let project: any = null;
    const action = vi.fn(async (a: string, p: any) => {
      if (a === "list") return project ? [project] : [];
      if (a === "create")
        return (project = {
          id: "p",
          title: p.title,
          revision: 0,
          documents: [],
          units: [],
          entries: [],
          publications: [],
        });
      return project;
    });
    render(
      <SourceLibrary
        port={{ sourceAction: action } as unknown as DesktopPort}
      />,
    );
    fireEvent.change(screen.getByLabelText("资料项目名称"), {
      target: { value: "Cambridge Academic" },
    });
    fireEvent.click(screen.getByText("新建资料项目"));
    await screen.findByText("补充题册 / 答案 / 解析 / 音频");
    expect(
      screen.getByRole("heading", { name: "Cambridge Academic" }),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("答案")).toBeInTheDocument();
    expect(screen.getByLabelText("解析")).toBeInTheDocument();
  });
  it("shows missing answer and conflict counts instead of a generic success message", async () => {
    const project = {
      id: "p",
      title: "Book",
      revision: 1,
      documents: [],
      units: [
        {
          id: "answers",
          role: "answers",
          state: "complete",
          test: "Test 1",
          section: "reading",
          part: 1,
          confirmed: true,
          start: 0,
          end: 0,
          result: { section: { questions: null }, entries: 1 },
        },
        {
          id: "u",
          title: "Reading",
          test: "Test 1",
          role: "questions",
          section: "reading",
          part: 1,
          state: "complete",
          confirmed: true,
          result: {
            section: { questions: [{ id: "q", number: 1, conflicts: [{}] }] },
          },
        },
      ],
      entries: [
        {
          id: "e",
          state: "unassigned",
          kind: "explanations",
          quote: "explanation",
          numbers: [1],
          test: "Test 1",
          section: "reading",
        },
      ],
      publications: [],
    };
    render(
      <SourceLibrary
        port={
          {
            sourceAction: async (a: string) =>
              a === "list" ? [project] : project,
          } as unknown as DesktopPort
        }
      />,
    );
    fireEvent.click(await screen.findByText("Book"));
    await screen.findByText(/缺答案 1/);
    expect(screen.getByText(/冲突 1/)).toBeInTheDocument();
    expect(screen.getByText("待确认匹配")).toBeInTheDocument();
  });
});
