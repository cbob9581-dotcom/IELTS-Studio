import {test,expect} from "vitest";
import * as T from "three";
import {configureExaminerMesh} from "./examinerMaterials";
test("eyelash cards cannot cast opaque rectangular shadows over the eyes",()=>{
 const material=new T.MeshStandardMaterial({transparent:true,opacity:1});material.name="Examiner_Body.eyelashes01.002";
 const mesh=new T.Mesh(new T.PlaneGeometry(),material);mesh.name="P2_Examiner_Eyelashes";mesh.castShadow=true;
 configureExaminerMesh(mesh);
 expect(mesh.castShadow).toBe(false);expect(mesh.receiveShadow).toBe(false);
 expect(material.transparent).toBe(true);expect(material.alphaTest).toBeGreaterThan(0);expect(material.depthWrite).toBe(false);
});
