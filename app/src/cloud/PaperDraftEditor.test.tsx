import {test,expect,vi} from "vitest";
import {render,screen,fireEvent} from "@testing-library/react";
import {fixturePaper} from "../../test/fixtures/paper";
import {PaperDraftEditor} from "./PaperDraftEditor";
test("human review edits question wording without changing source or identity",()=>{
 const p=fixturePaper(),change=vi.fn();render(<PaperDraftEditor paper={p} onChange={change}/>);
 fireEvent.change(screen.getByLabelText("题干"),{target:{value:"Corrected source wording"}});
 const next=change.mock.calls[0][0];expect(next.sections[0].questions[0].prompt).toBe("Corrected source wording");expect(next.id).toBe(p.id);expect(next.provenance).toEqual(p.provenance);expect(next.version).toBe(p.version);
});
