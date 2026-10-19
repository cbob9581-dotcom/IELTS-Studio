import {test,expect} from "vitest";
import {noiseEvent} from "./noise";
test("ambient schedule is repeatable, respects enabled sounds, and has bounded intervals",()=>{
 const a=noiseEvent("attempt-a",0,["keyboard","handwriting"],"light");
 expect(noiseEvent("attempt-a",0,["keyboard","handwriting"],"light")).toEqual(a);
 expect(a!.delayMs).toBeGreaterThanOrEqual(8000);expect(a!.delayMs).toBeLessThanOrEqual(24000);
 expect(noiseEvent("attempt-a",2,["keyboard"],"challenge")!.kind).toBe("keyboard");
 expect(noiseEvent("attempt-a",2,[],"light")).toBeNull();expect(noiseEvent("attempt-a",2,["keyboard"],"clean")).toBeNull();
});
