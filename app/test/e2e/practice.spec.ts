import { test, expect } from "@playwright/test";
test("library practice does not inherit exam mode from home", async ({
  page,
}) => {
  await page.goto("/");
  await page.getByRole("button", { name: /考试模式/ }).click();
  await page.getByRole("button", { name: "我的题库" }).click();
  await page.getByRole("button", { name: "进入考场" }).first().click();
  await expect(page.getByRole("region", { name: "三维考场" })).toBeVisible();
});
test("practice survives reload and submits into history", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: /训练模式/ }).click();
  await page.getByRole("button", { name: "阅读", exact: true }).click();
  await page.getByRole("button", { name: "进入考场" }).first().click();
  await page.getByRole("region", { name: "三维考场", exact:true }).focus();
  await page.keyboard.press("e");
  await page.getByRole("textbox", { name: "第 1 题答案" }).fill("solar");
  await expect(page.getByText("已保存", { exact: true })).toBeVisible();
  await page.reload();
  await page.getByRole("button", { name: "学习记录" }).click();
  await page.getByRole("button", { name: "继续练习" }).first().click();
  await expect(page.getByRole("textbox", { name: "第 1 题答案" })).toHaveValue(
    "solar",
  );
  await page.getByRole("button", { name: "提交作答" }).click();
  await page.getByRole("button", { name: "确认提交" }).click();
  await expect(page.getByRole("heading", { name: "练习报告" })).toBeVisible();
  await expect(page.getByText("1 / 1", { exact: true })).toBeVisible();
  await page.screenshot({
    path: "test-results/practice-report.png",
    fullPage: true,
  });
});
