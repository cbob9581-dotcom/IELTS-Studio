import { expect, test,vi } from "vitest";
import * as T from "three";
import { clampPaperPoint, wrapPaperText,createPaperSurface } from "./paperSurfaces";
test("unchanged note content does not upload a two-megapixel texture on every clock tick",()=>{
 const ctx=new Proxy({},{get:()=>()=>{}});
 const spy=vi.spyOn(HTMLCanvasElement.prototype,"getContext").mockReturnValue(ctx as never);
 try{const ink=createPaperSurface(new T.Object3D(),.1,.2),texture=(ink.surface.material as T.MeshStandardMaterial).map!;
 ink.draw("",{text:"",strokes:[]});const version=texture.version;
 ink.draw("",{text:"",strokes:[]});expect(texture.version).toBe(version);
 ink.draw("",{text:"new note",strokes:[]});expect(texture.version).toBeGreaterThan(version);
 }finally{spy.mockRestore();}
});
test("paper coordinates are bounded and invalid pointer input is rejected",()=>{
 expect(clampPaperPoint(-0.2,1.4)).toEqual({x:0,y:1});
 expect(clampPaperPoint(NaN,0.5)).toBeNull();
});
test("wrapping preserves words and explicit lines without cropping long words",()=>{
 expect(wrapPaperText("one two three\nfour",7)).toEqual(["one two","three","four"]);
 expect(wrapPaperText("abcdefghij",4)).toEqual(["abcd","efgh","ij"]);
});
