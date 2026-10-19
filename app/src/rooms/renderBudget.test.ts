import {test,expect} from "vitest";
import {shouldDraw} from "./renderBudget";
test("hidden rooms do not draw; covered rooms limit work without resetting scene",()=>{
 expect(shouldDraw(1000,0,false,true,true)).toBe(false);
 expect(shouldDraw(10,0,true,false,false)).toBe(false);
 expect(shouldDraw(70,0,true,false,false)).toBe(true);
 expect(shouldDraw(34,0,true,false,true)).toBe(true);
 expect(shouldDraw(5,0,true,true,true)).toBe(true);
});
