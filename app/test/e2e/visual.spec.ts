import {test,expect} from '@playwright/test';
test('long article and diagram remain usable in a narrow window',async({page})=>{
 await page.setViewportSize({width:1000,height:800});await page.goto('/test/e2e/visual-exam.html');
 await expect(page.getByRole('img',{name:'题目标注图'})).toBeVisible();await page.getByText(/END OF LONG ARTICLE/).scrollIntoViewIfNeeded();await expect(page.getByText(/END OF LONG ARTICLE/)).toBeVisible();
 await page.getByRole('textbox',{name:'第 1 题答案'}).fill('solar');await page.getByRole('textbox',{name:'标注 A'}).fill('Wednesday');
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
 await page.screenshot({path:'../docs/previews/reading-long-narrow.png',fullPage:true});
});
test('exam view does not expose training answer controls',async({page})=>{
 await page.goto('/test/e2e/visual-exam.html?mode=exam');await expect(page.getByRole('button',{name:'提交作答'})).toBeVisible();await expect(page.getByRole('button',{name:'查看解析'})).toHaveCount(0);await expect(page.getByText('参考答案',{exact:true})).toHaveCount(0);
});
