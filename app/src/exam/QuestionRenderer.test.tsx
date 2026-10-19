import { render, screen, fireEvent } from "@testing-library/react";
import { test, expect } from "vitest";
import { QuestionRenderer } from "./QuestionRenderer";
import { wordCount } from "./WritingEditor";
import { fixturePaper } from "../../test/fixtures/paper";
test("TFNG offers TRUE/FALSE/NOT GIVEN while YNNG offers YES/NO/NOT GIVEN", () => {
  const q = fixturePaper().sections[0].questions[1];
  const { rerender } = render(
    <QuestionRenderer question={q} value="" onChange={() => {}} />,
  );
  expect(screen.getByRole("radio", { name: "TRUE" })).toBeInTheDocument();
  expect(screen.queryByRole("radio", { name: "YES" })).toBeNull();
  rerender(
    <QuestionRenderer
      question={{ ...q, kind: "yes_no_not_given" }}
      value=""
      onChange={() => {}}
    />,
  );
  expect(screen.getByRole("radio", { name: "YES" })).toBeInTheDocument();
  expect(screen.queryByRole("radio", { name: "TRUE" })).toBeNull();
});
test("multiple choice reflects selected values and emits changed set", () => {
  const q = {
    ...fixturePaper().sections[0].questions[0],
    kind: "multiple_choice" as const,
    options: ["A", "B"],
  };
  let changed: unknown;
  render(
    <QuestionRenderer
      question={q}
      value={["A"]}
      onChange={(v) => (changed = v)}
    />,
  );
  expect(screen.getByRole("checkbox", { name: "A" })).toBeChecked();
  fireEvent.click(screen.getByRole("checkbox", { name: "B" }));
  expect(changed).toEqual(["A", "B"]);
});
test("word count ignores whitespace", () => {
  expect(wordCount("   \n  ")).toBe(0);
  expect(wordCount("one  two\nthree")).toBe(3);
});
test('matching selections round-trip as slot values',()=>{
 const q={...fixturePaper().sections[0].questions[0],kind:'matching' as const,slots:['Library','School'],options:['A','B']};let value:unknown={};
 const {rerender}=render(<QuestionRenderer question={q} value={{}} onChange={v=>value=v}/>);
 fireEvent.change(screen.getByRole('combobox',{name:'Library'}),{target:{value:'B'}});
 rerender(<QuestionRenderer question={q} value={value as Record<string,string>} onChange={v=>value=v}/>);
 expect(screen.getByRole('combobox',{name:'Library'})).toHaveValue('B');expect(screen.getByRole('combobox',{name:'School'})).toHaveValue('');
});
