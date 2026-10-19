import {test,expect} from "@playwright/test";
test("real WAV clips play continuously and finished state survives reload",async({page})=>{
 await page.goto("/test/e2e/listening-flow.html");
 await page.getByRole("button",{name:"播放音频 1/2"}).click();
 await expect(page.getByRole("button",{name:"听力播放已完成"})).toBeDisabled();
 expect(await page.evaluate(()=>JSON.parse(localStorage.getItem("listening-position")??"{}").track)).toBe(1);
 await page.reload();await expect(page.getByRole("button",{name:"听力播放已完成"})).toBeDisabled();
});
