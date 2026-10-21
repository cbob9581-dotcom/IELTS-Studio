import * as T from "three";
export type ExaminerMotion={stage:string;mouth:number;handoff:boolean;articulation?:{rounding:number;width:number}};
export function createExaminerAnimation(root:T.Object3D,random= Math.random){
 const controlled=["spine02","neck01","head","eye.L","eye.R","upperarm01.L","lowerarm01.L","wrist.L","upperarm01.R","lowerarm01.R","wrist.R"];
 const nodes=new Map<string,{node:T.Object3D;before:T.Quaternion;changed:boolean}>();
 root.traverse(node=>{const name=controlled.find(name=>node.name===name||node.name===T.PropertyBinding.sanitizeNodeName(name));if(name)nodes.set(name,{node,before:node.quaternion.clone(),changed:false});});
 const channels=["eyeBlinkLeft","eyeBlinkRight","jawOpen","mouthFunnel","mouthPucker","mouthStretchLeft","mouthStretchRight","mouthSmileLeft","mouthSmileRight","browInnerUp","browOuterUpLeft","browOuterUpRight","eyeSquintLeft","eyeSquintRight","eyeLookDownLeft","eyeLookDownRight"];
 const faces:{weights:number[];slots:{name:string;index:number;before:number}[]}[]=[];
 root.traverse(o=>{if(o instanceof T.Mesh&&o.morphTargetDictionary&&o.morphTargetInfluences){const weights=o.morphTargetInfluences;faces.push({weights,slots:channels.flatMap(name=>{const i=o.morphTargetDictionary![name];return i===undefined?[]:[{name,index:i,before:weights[i]}];})});}});
 const parentQ=new T.Quaternion(),inverseQ=new T.Quaternion(),offset=new T.Quaternion(),euler=new T.Euler(0,0,0,"XYZ");
 const rotate=(name:string,x:number,y=0,z=0)=>{const item=nodes.get(name);if(!item)return;
   const {node,before}=item;before.copy(node.quaternion);item.changed=true;
   node.parent?.getWorldQuaternion(parentQ);if(!node.parent)parentQ.identity();inverseQ.copy(parentQ).invert();
   offset.setFromEuler(euler.set(x,y,z));offset.premultiply(inverseQ).multiply(parentQ);node.quaternion.premultiply(offset);
 };
 let time=0,stage="",nextBlink=2+random()*3,blinkStarted=-10,nextGlance=3+random()*5,glanceStarted=-10,glanceSide=1;
 let nextNod=2+random()*4,nodStarted=-10,nextGesture=1+random()*2,gestureStarted=-10,gestureSide="L";
 let jaw=0,activity=0,preparing=0,rounding=.3,width=0;
 const envelope=(t:number,duration:number)=>t<0||t>=duration?0:Math.sin(Math.PI*t/duration)**2;
 return {
 restore(){for(const item of nodes.values())if(item.changed){item.node.quaternion.copy(item.before);item.changed=false;}for(const face of faces)for(const s of face.slots)face.weights[s.index]=s.before;},
 update(delta:number,motion:ExaminerMotion){
   const dt=Math.min(Math.max(delta,0),.1);time+=dt;
   const listening=["answering","long_turn","roundoff_answer"].includes(motion.stage);
   const speaking=["greeting","question","roundoff_question","begin_long_turn","closing"].includes(motion.stage)&&motion.mouth>.025;
   if(stage!==motion.stage){stage=motion.stage;if(listening)nextNod=time+2+random()*4;if(speaking)nextGesture=time+.4+random()*1.3;}
   const ease=1-Math.exp(-dt*9);activity+=((speaking?1:0)-activity)*ease;preparing+=((stage==="preparation"?1:0)-preparing)*ease;
   jaw+=(T.MathUtils.clamp(motion.mouth,0,1)*.28-jaw)*(1-Math.exp(-dt*25));
   rounding+=((motion.articulation?.rounding??.3)-rounding)*(1-Math.exp(-dt*18));width+=((motion.articulation?.width??0)-width)*(1-Math.exp(-dt*18));
   if(time>=nextBlink){blinkStarted=time;nextBlink=time+2.8+random()*4.5;}
   if(time>=nextGlance){glanceStarted=time;glanceSide=random()<.5?-1:1;nextGlance=time+5+random()*7;}
   if(listening&&time>=nextNod){nodStarted=time;nextNod=time+5+random()*7;}
   if(speaking&&!motion.handoff&&time>=nextGesture){gestureStarted=time;gestureSide=random()<.65?"L":"R";nextGesture=time+4+random()*5;}
   const blink=envelope(time-blinkStarted,.19)*.94,glance=envelope(time-glanceStarted,1.7),nod=envelope(time-nodStarted,.95)*.035;
   // World axes keep the small overlays consistent across the imported bone orientations.
   if(!motion.handoff){
     rotate("spine02",Math.sin(time*1.65)*.003);
     rotate("neck01",nod*.3+preparing*.016,Math.sin(time*.48)*.01,Math.sin(time*.71)*.006);
     rotate("head",nod*.7+preparing*.025+glance*.008,Math.sin(time*.38)*.02+glance*glanceSide*.028,Math.sin(time*.63)*.011);
     const gesture=envelope(time-gestureStarted,2.4)*activity;
     rotate(`upperarm01.${gestureSide}`,-gesture*.025);
     rotate(`lowerarm01.${gestureSide}`,-gesture*.14,0,gesture*.018*(gestureSide==="L"?1:-1));
     rotate(`wrist.${gestureSide}`,0,gesture*.065,gesture*.08*(gestureSide==="L"?1:-1));
   }
   rotate("eye.L",preparing*.055+glance*.025,glance*glanceSide*.035);
   rotate("eye.R",preparing*.055+glance*.025,glance*glanceSide*.035);
   const smile=stage==="greeting"||stage==="closing"?.055:listening?.018:0;
   const expression:Record<string,number>={eyeBlinkLeft:blink,eyeBlinkRight:blink*.98,jawOpen:jaw,
     mouthFunnel:jaw*(.08+rounding*.5),mouthPucker:jaw*rounding*.18,mouthStretchLeft:width*.08,mouthStretchRight:width*.075,mouthSmileLeft:smile,mouthSmileRight:smile*.9,
     browInnerUp:activity*.035,browOuterUpLeft:activity*.025,browOuterUpRight:activity*.018,
     eyeSquintLeft:listening?.015:0,eyeSquintRight:listening?.015:0,
     eyeLookDownLeft:preparing*.08,eyeLookDownRight:preparing*.08};
   for(const face of faces)for(const slot of face.slots)face.weights[slot.index]=Math.min(1,slot.before+(expression[slot.name]??0));
 }
 };
}
