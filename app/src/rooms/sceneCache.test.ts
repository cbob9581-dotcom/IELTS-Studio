import {test,expect,vi} from 'vitest';
import * as T from 'three';
import {createSceneCache} from './sceneCache';
test('parsed scene is reused while poses/materials are isolated and shared geometry survives release',async()=>{
 const geometry=new T.BoxGeometry(),material=new T.MeshStandardMaterial(),texture=new T.Texture();material.map=texture;
 const scene=new T.Group();scene.add(new T.Mesh(geometry,material));const load=vi.fn(async()=>({scene,animations:[],cameras:[]}));
 const cache=createSceneCache(load,10000);const a=await cache.acquire('room'),b=await cache.acquire('room');
 expect(load).toHaveBeenCalledTimes(1);const ma=a.scene.children[0] as T.Mesh,mb=b.scene.children[0] as T.Mesh;
 expect(ma.geometry).not.toBe(mb.geometry);expect(ma.geometry.attributes.position.array).toBe(mb.geometry.attributes.position.array);expect(ma.material).not.toBe(mb.material);ma.position.x=5;expect(mb.position.x).toBe(0);
 const instanceDisposed=vi.fn();ma.geometry.addEventListener('dispose',instanceDisposed);
 const disposed=vi.fn();geometry.addEventListener('dispose',disposed);a.release();a.release();expect(disposed).not.toHaveBeenCalled();expect(instanceDisposed).toHaveBeenCalledTimes(1);
 cache.clear();expect(disposed).not.toHaveBeenCalled();b.release();expect(disposed).toHaveBeenCalledTimes(1);
});
test('new model keys retire old cache entries without freeing an active instance',async()=>{
 const load=vi.fn(async()=>({scene:new T.Group(),animations:[],cameras:[]}));const cache=createSceneCache(load,10000);
 const a=await cache.acquire('one');const b=await cache.acquire('two');expect(load).toHaveBeenCalledTimes(2);a.release();b.release();cache.clear();
});
