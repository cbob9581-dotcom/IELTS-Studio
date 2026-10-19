import type {DesktopPort} from "../platform/desktop";
const cache=new WeakMap<DesktopPort,{key:string;bytes:Promise<ArrayBuffer>}>();
export function loadRuntimeModel(port:DesktopPort,key:string){
 let entry=cache.get(port);
 if(!entry||entry.key!==key){const bytes=port.readRuntimeModel();entry={key,bytes};cache.set(port,entry);
  bytes.catch(()=>{if(cache.get(port)?.bytes===bytes)cache.delete(port);});
 }
 return entry.bytes;
}
