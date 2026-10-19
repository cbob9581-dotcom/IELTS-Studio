import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { test, expect } from "vitest";
import { ConfigurationPanel } from "./ConfigurationPanel";
import { defaultConfig } from "./types";
import type { DesktopPort } from "../platform/desktop";
test("edits provider model and saves ordinary settings without password fields", async () => {
  let saved: unknown = null;
  const port = {
    getConfig: async () => defaultConfig(),
    credentialStatus: async () => ({}),
    saveConfig: async (c: unknown) => {
      saved = c;
    },
    chooseFile: async () => null,
  } as unknown as DesktopPort;
  render(<ConfigurationPanel port={port} />);
  const model = await screen.findByRole("textbox", { name: "DeepSeek 模型" });
  fireEvent.change(model, { target: { value: "custom-model" } });
  fireEvent.click(screen.getByRole("button", { name: "保存全部配置" }));
  await waitFor(() =>
    expect(
      (saved as ReturnType<typeof defaultConfig>).providers.deepseek.model,
    ).toBe("custom-model"),
  );
  expect(JSON.stringify(saved)).not.toContain("apiKey");
  expect(screen.getByLabelText("DeepSeek API 密钥")).toHaveAttribute(
    "type",
    "password",
  );
});
