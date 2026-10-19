import * as T from "three";
export function configureExaminerMesh(mesh:T.Mesh):void{
  const materials=Array.isArray(mesh.material)?mesh.material:[mesh.material];
  mesh.receiveShadow=true;mesh.castShadow=true;
  for(const material of materials){
    const name=material.name.toLowerCase();
    if(!(material instanceof T.MeshStandardMaterial))continue;
    if(material.alphaTest>0&&!material.transparent)material.alphaToCoverage=true;
    material.envMapIntensity=.45;
    const fineCard=name.includes("eyelash")||name.includes("eyebrow");
    if(fineCard){
      mesh.castShadow=false;
      mesh.receiveShadow=false;
      material.transparent=true;material.alphaTest=.015;material.depthWrite=false;
      // The image's alpha masks the fine fibers. A solid shadow proxy must never cover the socket.
    }
    if(name.includes("groomstrand")){mesh.castShadow=false;mesh.receiveShadow=false;}
    if(name.includes("rebuilthair"))mesh.receiveShadow=false;
    if(name.includes("high-poly")){mesh.castShadow=false;mesh.receiveShadow=false;material.transparent=false;material.alphaTest=.02;material.depthWrite=true;}
  }
}
