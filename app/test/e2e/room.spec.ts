import { test, expect } from "@playwright/test";
test("room E shortcut opens paper and typing E does not navigate away", async ({
  page,
}) => {
  await page.goto("/");
  await page.getByRole("button", { name: /训练模式/ }).click();
  await page.getByRole("button", { name: "阅读", exact: true }).click();
  await page.getByRole("checkbox", { name: "显示开发测试素材" }).check();
  await page.getByRole("button", { name: "进入考场" }).first().click();
  const room = page.getByRole("region", { name: "三维考场" });
  await expect(room).toBeVisible();
  await room.focus();
  await page.keyboard.press("e");
  const input = page.getByRole("textbox", { name: "第 1 题答案" });
  await expect(input).toBeVisible();
  await input.fill("e");
  await expect(input).toHaveValue("e");
});
