import { render, screen, fireEvent } from "@testing-library/react";
import { test, expect } from "vitest";
import App from "./App";
test("home offers Academic modes and all four sections with truthful cloud status", () => {
  render(<App />);
  expect(screen.getAllByText("Academic").length).toBeGreaterThan(0);
  expect(screen.getByRole("button", { name: /考试模式/ })).toBeInTheDocument();
  expect(screen.getByRole("button", { name: /训练模式/ })).toBeInTheDocument();
  for (const section of ["听力", "阅读", "写作", "口语"])
    expect(screen.getAllByText(section).length).toBeGreaterThan(0);
  expect(screen.getByText(/云服务配置与接入状态/)).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: /训练模式/ }));
  expect(screen.getByRole("heading", { name: /选择练习/ })).toBeInTheDocument();
});
test("opening library after exam selection returns to training mode", () => {
  render(<App />);
  fireEvent.click(screen.getByRole("button", { name: /考试模式/ }));
  fireEvent.click(screen.getByRole("button", { name: "我的题库" }));
  expect(screen.getByRole("heading", { name: "选择练习" })).toBeInTheDocument();
});
