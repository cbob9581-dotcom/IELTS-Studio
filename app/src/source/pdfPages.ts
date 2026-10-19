import type {DesktopPort} from '../platform/desktop';
type Cached={task:import('pdfjs-dist').PDFDocumentLoadingTask;pdf:import('pdfjs-dist').PDFDocumentProxy};
const docs=new WeakMap<DesktopPort,Map<string,Promise<Cached>>>();
export async function prepareSourcePages(port:DesktopPort,id:string,documentId:string,pages:number[]){
 if(!port.sourceAsset||!port.sourceAction)throw Error('原件预览未连接');
 let project=await port.sourceAction('read',{id});let d=project.documents.find((x:{id:string})=>x.id===documentId);
 if(!d||d.extension!=='pdf')return;
 const needed=[...new Set(pages)].filter(p=>!d.pageImages?.[String(p)]||d.pageTexts?.[String(p)]===undefined);if(!needed.length)return;
 let cache=docs.get(port);if(!cache){cache=new Map();docs.set(port,cache);}const key=`${id}/${documentId}`;
 let pending=cache.get(key);
 if(!pending){pending=(async()=>{const lib=await import('pdfjs-dist');lib.GlobalWorkerOptions.workerSrc=new URL('pdfjs-dist/build/pdf.worker.min.mjs',import.meta.url).toString();const bytes=await port.sourceAsset!(id,documentId,null);const task=lib.getDocument({data:new Uint8Array(bytes),enableXfa:false,useSystemFonts:true});return {task,pdf:await task.promise};})();cache.set(key,pending);}
 const {pdf}=await pending;
 for(const n of needed){
  const page=await pdf.getPage(n);const original=page.getViewport({scale:1});const viewport=page.getViewport({scale:1400/original.width});
  const canvas=document.createElement('canvas');canvas.width=Math.ceil(viewport.width);canvas.height=Math.ceil(viewport.height);
  await page.render({canvas,viewport}).promise;
  const content=await page.getTextContent();const pageText=content.items.map(item=>'str' in item?item.str+(item.hasEOL?'\n':' '):'').join('');
  const blob=await new Promise<Blob>((resolve,reject)=>canvas.toBlob(b=>b?resolve(b):reject(Error('原页题图导出失败')),'image/png'));
  const bytes=new Uint8Array(await blob.arrayBuffer());let binary='';for(let i=0;i<bytes.length;i+=32768)binary+=String.fromCharCode(...bytes.subarray(i,i+32768));
  project=await port.sourceAction('read',{id});await port.sourceAction('page_image',{id,revision:project.revision,targetId:documentId,page:n,pngBase64:btoa(binary),pageText});
  canvas.width=0;canvas.height=0;page.cleanup();
 }
}
export function releaseSourcePages(port:DesktopPort){const cache=docs.get(port);docs.delete(port);if(cache)for(const pending of cache.values())void pending.then(v=>v.task.destroy()).catch(()=>{});}
