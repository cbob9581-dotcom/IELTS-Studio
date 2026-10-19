import {test,expect} from "@playwright/test";
test("finishing the speaking session releases the real captured media track",async({page})=>{
 test.setTimeout(45000);await page.goto("/test/e2e/speaking-room.html?finish");
 await expect.poll(()=>page.evaluate(()=>JSON.parse(localStorage.getItem("speaking-fixture")??"{}").status),{timeout:30000}).toBe("submitted");
 await expect.poll(async()=>{
   await page.locator("#track-state").evaluate((el:HTMLButtonElement)=>el.click());
   return page.locator("#track-state").getAttribute("data-states");
 }).toBe("ended");
});
test("missing cloud configuration never starts a silent examination or microphone",async({page})=>{
 test.setTimeout(45000);await page.goto("/test/e2e/speaking-room.html?missing-service");
 await expect(page.getByText(/请回主页设置 DeepSeek/)).toBeVisible({timeout:30000});
 await expect(page.locator(".test-stage")).toHaveText("loading");
 await page.locator("#track-state").evaluate((el:HTMLButtonElement)=>el.click());
 await expect(page.locator("#track-state")).toHaveAttribute("data-states","");
});
