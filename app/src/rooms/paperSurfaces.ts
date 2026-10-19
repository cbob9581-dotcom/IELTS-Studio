import * as T from "three";
export type PaperNotes={text:string;strokes:{x:number;y:number;start?:boolean}[]};
export const emptyNotes=():PaperNotes=>({text:"",strokes:[]});
export function clampPaperPoint(x:number,y:number):{x:number;y:number}|null {
  return Number.isFinite(x)&&Number.isFinite(y)?{x:Math.max(0,Math.min(1,x)),y:Math.max(0,Math.min(1,y))}:null;
}
export function wrapPaperText(text:string,columns:number):string[]{
  const result:string[]=[];
  for(const paragraph of text.split("\n")){
    let line="";
    for(let word of paragraph.split(/\s+/).filter(Boolean)){
      if(line&&(line+" "+word).length>columns){result.push(line);line="";}
      while(word.length>columns){if(line){result.push(line);line="";}result.push(word.slice(0,columns));word=word.slice(columns);}
      if(word)line+=(line?" ":"")+word;
    }
    result.push(line);
  }
  return result;
}
export function createPaperSurface(parent:T.Object3D,width:number,height:number){
  let lastText:string|undefined,lastNotes:PaperNotes|null|undefined;
  const sameNotes=(a:PaperNotes|null|undefined,b:PaperNotes|null)=>a===b||!!a&&!!b&&a.text===b.text&&a.strokes.length===b.strokes.length&&a.strokes.every((p,i)=>p.x===b.strokes[i].x&&p.y===b.strokes[i].y&&!!p.start===!!b.strokes[i].start);
  const canvas=document.createElement("canvas");canvas.width=1200;canvas.height=1700;
  const texture=new T.CanvasTexture(canvas);texture.colorSpace=T.SRGBColorSpace;texture.anisotropy=8;
  const surface=new T.Mesh(new T.PlaneGeometry(width,height),new T.MeshStandardMaterial({map:texture,roughness:.98,side:T.FrontSide}));
  surface.name="IELTS_InteractivePaper";surface.rotation.x=-Math.PI/2;surface.position.y=.0016;
  parent.add(surface);
  const draw=(text:string,notes:PaperNotes|null)=>{
    if(lastText===text&&sameNotes(lastNotes,notes))return;
    lastText=text;lastNotes=notes;
    const c=canvas.getContext("2d")!;c.fillStyle="#faf8ef";c.fillRect(0,0,1200,1700);
    c.fillStyle="#292d2b";
    c.font="bold 44px Georgia";c.fillText(notes?"NOTES":"IELTS · SPEAKING PART 2",85,105);
    c.strokeStyle="#bab9b1";c.lineWidth=2;c.beginPath();c.moveTo(85,138);c.lineTo(1115,138);c.stroke();
    const lines=wrapPaperText(notes?notes.text:text,40);
    const font=Math.min(notes?38:44,1420/Math.max(lines.length,1)/1.45);
    c.font=`${font}px Georgia`;lines.forEach((line,i)=>c.fillText(line,85,205+i*font*1.45));
    if(notes){
      c.lineWidth=3;c.strokeStyle="#35435c";c.lineCap="round";c.beginPath();
      notes.strokes.forEach((p,i)=>{const x=p.x*1200,y=p.y*1700;if(p.start||!i)c.moveTo(x,y);else c.lineTo(x,y);});c.stroke();
    }
    texture.needsUpdate=true;
  };
  return {surface,draw};
}
