import {test,expect} from "vitest";
import * as T from "three";
import {createExaminerAnimation} from "./examinerAnimation";
function fixture(){const root=new T.Group();for(const name of ["spine02","neck01","head","lowerarm01.L","wrist.L","eye.L","eye.R"]){const b=new T.Bone();b.name=name;root.add(b);}const face=new T.Mesh();face.morphTargetDictionary={jawOpen:0,eyeBlinkLeft:1,eyeBlinkRight:2,mouthSmileLeft:3,browInnerUp:4};face.morphTargetInfluences=[0,0,0,.015,0];root.add(face);return {root,face,head:root.getObjectByName("head")!,arm:root.getObjectByName("lowerarm01.L")!};}
test("listening has restrained motion and restores the authored pose without accumulating offsets",()=>{
 const {root,head}=fixture(),animation=createExaminerAnimation(root,()=>.5),base=head.quaternion.clone();
 for(let i=0;i<180;i++){animation.restore();animation.update(1/60,{stage:"answering",mouth:0,handoff:false});}
 expect(head.quaternion.angleTo(base)).toBeGreaterThan(.001);expect(head.quaternion.angleTo(base)).toBeLessThan(.12);
 animation.restore();expect(head.quaternion.angleTo(base)).toBeLessThan(1e-6);
});
test("a speaking gesture releases arm control throughout the authored handoff",()=>{
 const {root,arm}=fixture(),animation=createExaminerAnimation(root,()=>.5),base=arm.quaternion.clone();let max=0;
 for(let i=0;i<360;i++){animation.restore();animation.update(1/60,{stage:"question",mouth:.6,handoff:false});max=Math.max(max,arm.quaternion.angleTo(base));}
 expect(max).toBeGreaterThan(.02);expect(max).toBeLessThan(.3);
 animation.restore();animation.update(1/60,{stage:"instructions",mouth:.4,handoff:true});expect(arm.quaternion.angleTo(base)).toBeLessThan(1e-6);
});
test("speech eases the jaw and preserves character morphs on restoration",()=>{
 const {root,face}=fixture(),animation=createExaminerAnimation(root,()=>.5);
 animation.update(.016,{stage:"question",mouth:1,handoff:false});expect(face.morphTargetInfluences![0]).toBeGreaterThan(0);expect(face.morphTargetInfluences![0]).toBeLessThan(.28);
 animation.restore();expect(face.morphTargetInfluences).toEqual([0,0,0,.015,0]);
});
