import { render, screen, fireEvent } from "@testing-library/react";
import { test, expect } from "vitest";
import { Settings } from "./Settings";
import type { DesktopPort } from "../platform/desktop";
import { defaultConfig } from "../configuration/types";
test("changing existing roots requires reviewable migration confirmation", () => {
  render(
    <Settings
      port={
        {
          getConfig: async () => defaultConfig(),
          credentialStatus: async () => ({}),
        } as unknown as DesktopPort
      }
      roots={{ assets: "D:/old-assets", study: "D:/old-study" }}
      onRoots={() => {}}
    />,
  );
  fireEvent.change(screen.getByRole("textbox", { name: "资产库目录" }), {
    target: { value: "D:/new-assets" },
  });
  fireEvent.click(screen.getByRole("button", { name: "迁移到新位置" }));
  expect(screen.getByRole("dialog")).toHaveTextContent("D:/new-assets");
  expect(screen.getByRole("button", { name: "确认迁移" })).toBeInTheDocument();
});
