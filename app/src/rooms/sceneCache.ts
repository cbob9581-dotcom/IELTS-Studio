import * as T from 'three';
import {clone} from 'three/addons/utils/SkeletonUtils.js';
import {GLTFLoader} from 'three/addons/loaders/GLTFLoader.js';
import type {DesktopPort} from '../platform/desktop';
import {loadRuntimeModel} from './runtimeModel';
type Asset={scene:T.Group;animations:T.AnimationClip[];cameras:T.Camera[]};
type Entry={key:string;promise:Promise<Asset>;refs:number;retired:boolean;timer?:ReturnType<typeof setTimeout>;asset?:Asset};
export function createSceneCache(load:(key:string)=>Promise<Asset>,idleMs=120000){
 const entries=new Map<string,Entry>();
 function resources(root:T.Object3D){const geometry=new Set<T.BufferGeometry>(),materials=new Set<T.Material>(),textures=new Set<T.Texture>();root.traverse(o=>{if(o instanceof T.Mesh){geometry.add(o.geometry);for(const m of Array.isArray(o.material)?o.material:[o.material]){materials.add(m);for(const value of Object.values(m))if(value instanceof T.Texture)textures.add(value);}}});return {geometry,materials,textures};}
 function destroy(e:Entry){if(e.refs)return;e.retired=true;if(e.timer)clearTimeout(e.timer);if(entries.get(e.key)===e)entries.delete(e.key);if(e.asset){const r=resources(e.asset.scene);r.geometry.forEach(g=>g.dispose());r.materials.forEach(m=>m.dispose());r.textures.forEach(t=>{const data=t.source?.data as {close?:()=>void}|undefined;if(typeof data?.close==='function')data.close();t.dispose();});e.asset=undefined;}}
 function idle(e:Entry){if(e.refs)return;if(e.retired){destroy(e);return;}if(e.timer)clearTimeout(e.timer);e.timer=setTimeout(()=>destroy(e),idleMs);}
 function get(key:string){let e=entries.get(key);if(e){if(e.timer)clearTimeout(e.timer);return e;}for(const old of entries.values())if(old.key!==key){old.retired=true;if(!old.refs)destroy(old);}
  e={key,refs:0,retired:false,promise:Promise.resolve(null as unknown as Asset)};const created=e;e.promise=load(key).then(asset=>{created.asset=asset;idle(created);return asset;}).catch(error=>{if(entries.get(key)===created)entries.delete(key);throw error;});entries.set(key,e);return e;
 }
 return {
  async warm(key:string){await get(key).promise;},
  async acquire(key:string){const e=get(key);e.refs++;let asset:Asset;try{asset=await e.promise;}catch(error){e.refs--;throw error;}
   const shared=resources(asset.scene);let scene:T.Group;try{scene=clone(asset.scene) as T.Group;}catch(error){e.refs--;idle(e);throw error;}const materials=new Map<T.Material,T.Material>();
   const geometries=new Map<T.BufferGeometry,T.BufferGeometry>();
   const geometry=(source:T.BufferGeometry)=>{let g=geometries.get(source);if(g)return g;g=new T.BufferGeometry();g.name=source.name;g.setIndex(source.index);for(const [name,attribute]of Object.entries(source.attributes))g.setAttribute(name,attribute);g.morphAttributes=Object.fromEntries(Object.entries(source.morphAttributes).map(([key,list])=>[key,list?.slice()]));g.morphTargetsRelative=source.morphTargetsRelative;g.groups=source.groups.map(group=>({...group}));g.drawRange={...source.drawRange};g.boundingBox=source.boundingBox?.clone()??null;g.boundingSphere=source.boundingSphere?.clone()??null;geometries.set(source,g);return g;};
   const copy=(m:T.Material)=>{let result=materials.get(m);if(!result){result=m.clone();materials.set(m,result);}return result;};
   scene.traverse(o=>{if(o instanceof T.Mesh){o.geometry=geometry(o.geometry);o.material=Array.isArray(o.material)?o.material.map(copy):copy(o.material);}});const cameras:T.Camera[]=[];scene.traverse(o=>{if(o instanceof T.Camera)cameras.push(o);});let released=false;
   return {scene,animations:asset.animations,cameras,release(){if(released)return;released=true;const skeletons=new Set<T.Skeleton>();scene.traverse(o=>{if(o instanceof T.SkinnedMesh&&o.skeleton)skeletons.add(o.skeleton);});skeletons.forEach(s=>s.dispose());const r=resources(scene);r.materials.forEach(m=>m.dispose());r.geometry.forEach(g=>{if(!shared.geometry.has(g))g.dispose();});r.textures.forEach(t=>{if(!shared.textures.has(t))t.dispose();});scene.removeFromParent();e.refs--;idle(e);}};
  },
  clear(){for(const e of entries.values()){e.retired=true;if(!e.refs)destroy(e);}}
 };
}
const runtimes=new WeakMap<DesktopPort,ReturnType<typeof createSceneCache>>();
export function runtimeSceneCache(port:DesktopPort){let cache=runtimes.get(port);if(!cache){cache=createSceneCache(async key=>{const bytes=await loadRuntimeModel(port,key);return new GLTFLoader().parseAsync(bytes.slice(0),'');});runtimes.set(port,cache);}return cache;}
