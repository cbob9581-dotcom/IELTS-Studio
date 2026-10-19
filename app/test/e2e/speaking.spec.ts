import {test,expect} from "@playwright/test";
test("automatic handoff leads to readable 3D card and persistent paper notes",async({page})=>{
 test.setTimeout(90000);
 const errors:string[]=[];page.on("pageerror",e=>errors.push(e.message));
 await page.goto("/test/e2e/speaking-room.html");
 const room=page.getByRole("region",{name:"三维口语考场"});
 await expect(page.locator(".test-stage")).toHaveText("preparation",{timeout:65000});
 await page.waitForTimeout(3500);
 const canvas=await room.locator("canvas").elementHandle();
 await expect(page.getByRole("button",{name:/播放递卡|复位|打开桌面试卷/})).toHaveCount(0);
 await expect(page.locator(".exam")).toHaveCount(0);
 await page.screenshot({path:"../docs/previews/immersive-speaking-handoff.png"});
 await room.focus();await page.keyboard.press("e");
 await expect(room).toHaveAttribute("data-focus","card");
 await page.waitForTimeout(900);
 await page.screenshot({path:"../docs/previews/immersive-speaking-card.png"});
 // Camera is looking down at the card; the physical scratch paper is picked with a real pointer ray.
 // Use a bounded screen scan to find the real sheet, rather than a production test hook.
 await page.keyboard.press("Escape");
 await page.waitForTimeout(900);
 await room.focus();
 let found=false;
 for(let y=815;y<925&&!found;y+=25)for(let x=825;x<1060&&!found;x+=30){
   await page.mouse.click(x,y);found=await room.getAttribute("data-focus")==="notes";
 }
 expect(found).toBe(true);
 await page.waitForTimeout(900);
 await page.keyboard.type("quiet library - natural light");
 await page.keyboard.press("Home");await page.keyboard.press("End");await page.keyboard.type(" - notes");
 await expect(page.getByRole("textbox",{name:"草稿纸键盘输入"})).toHaveValue("quiet library - natural light - notes");
 await page.mouse.move(700,500);await page.mouse.down();await page.mouse.move(780,550,{steps:8});await page.mouse.up();
 expect(await page.evaluate(()=>JSON.parse(localStorage.getItem("speaking-fixture")!).speakingNotes.strokes.length)).toBeGreaterThan(1);
 await page.screenshot({path:"../docs/previews/immersive-speaking-notes.png"});
 await page.keyboard.press("Escape");expect(await canvas!.evaluate(el=>el.isConnected)).toBe(true);
 await page.reload();
 await expect(room.locator("canvas")).toBeVisible({timeout:45000});
 await expect(page.getByRole("textbox",{name:"草稿纸键盘输入"})).toHaveValue("quiet library - natural light - notes");
 expect(await page.evaluate(()=>JSON.parse(localStorage.getItem("speaking-fixture")!).speakingNotes.strokes.length)).toBeGreaterThan(1);
 expect(errors).toEqual([]);
});
