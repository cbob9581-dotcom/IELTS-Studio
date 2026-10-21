import { useEffect, useRef, useState } from "react";
import * as T from "three";
import {shouldDraw} from "./renderBudget";
import {runtimeSceneCache} from "./sceneCache";
import { RectAreaLightUniformsLib } from "three/addons/lights/RectAreaLightUniformsLib.js";
import fixtures from "../../../rooms/speaking/fixture-manifest.json";
import type { DesktopPort } from "../platform/desktop";
import { errorText } from "../platform/desktop";
import type { SpeakingState } from "../speaking/flow";
import { createPaperSurface, clampPaperPoint, emptyNotes, type PaperNotes } from "./paperSurfaces";
import type { MutableRefObject } from "react";
import {configureExaminerMesh} from "./examinerMaterials";
import {EffectComposer} from "three/addons/postprocessing/EffectComposer.js";
import {RenderPass} from "three/addons/postprocessing/RenderPass.js";
import {OutputPass} from "three/addons/postprocessing/OutputPass.js";
import {createExaminerAnimation} from "./examinerAnimation";
export function SpeakingRoom({port,flow,topic,notes,onNotes,onPropsReady,onReady,interactive=true,mouthLevel,articulation,renderScale=1,assetKey=""}:{
  port:DesktopPort;flow?:SpeakingState;topic:string;notes:PaperNotes;
  onNotes:(notes:PaperNotes)=>void;onPropsReady:()=>void;onReady:()=>void;
  interactive?:boolean;mouthLevel:MutableRefObject<number>;articulation?:MutableRefObject<import("../speaking/articulation").Articulation>;renderScale?:number;assetKey?:string;
}) {
  const host=useRef<HTMLDivElement>(null);
  const textInput=useRef<HTMLTextAreaElement>(null);
  const live=useRef({flow,topic,notes,onNotes,onPropsReady,onReady,interactive});
  live.current={flow,topic,notes,onNotes,onPropsReady,onReady,interactive};
  const [reload,setReload]=useState(0);
  const [status,setStatus]=useState("正在读取考官房间…");
  const [error,setError]=useState("");
  const [focus,setFocus]=useState<"room"|"card"|"notes">("room");
  const focusRef=useRef(focus);focusRef.current=focus;
  const controls=useRef<{lookUp:()=>void}|null>(null);
  useEffect(()=>{
    setError("");
    const el=host.current!;
    let cancel=false,frame=0;
    let renderer:T.WebGLRenderer|undefined,observer:ResizeObserver|undefined,mixer:T.AnimationMixer|undefined;
    let composer:EffectComposer|undefined;
    let model:T.Group|undefined;
    const scene=new T.Scene();let last=performance.now();
    let action:T.AnimationAction|undefined;let animating=false;
    const cleanup:(()=>void)[]=[];
    let releaseScene:(()=>void)|undefined;
    (async () => {
      try {
        renderer = new T.WebGLRenderer({
          antialias: false,
          powerPreference: "high-performance",
        });
        renderer.setPixelRatio(Math.min(devicePixelRatio * renderScale, 2));
        renderer.shadowMap.enabled = true;
        renderer.shadowMap.type = T.PCFShadowMap;
        renderer.toneMapping = T.AgXToneMapping;
        renderer.toneMappingExposure = 1;
        scene.background = new T.Color(0xe4e6df);
        el.appendChild(renderer.domElement);
        setStatus("正在准备材质与骨骼…");
        const gltf = await runtimeSceneCache(port).acquire(assetKey);
        if (cancel) {gltf.release();return;}
        releaseScene=gltf.release;
        model = gltf.scene;
        scene.add(model);
        model.updateMatrixWorld(true);
        let skins = 0,
          morphs = 0;
        model.traverse((o) => {
          if (o instanceof T.Light) o.visible = false;
          if (o instanceof T.Mesh) {
            configureExaminerMesh(o);
            if (o instanceof T.SkinnedMesh) skins++;
            if (o.morphTargetDictionary) {
              morphs = Math.max(
                morphs,
                Object.keys(o.morphTargetDictionary).length,
              );
            }
            for (const m of Array.isArray(o.material)
              ? o.material
              : [o.material]) {
              if (m instanceof T.MeshStandardMaterial) {
                m.envMapIntensity = 0.45;
              }
            }
          }
        });
        el.dataset.skins = String(skins);
        el.dataset.morphs = String(morphs);
        el.dataset.animations = String(gltf.animations.length);
        if (!skins) throw Error("场景未包含可用骨骼，请检查 GLB 导出。");
        const source =
          gltf.cameras.find((c) => c.name.includes("CandidateCamera")) ??
          gltf.cameras[0];
        const camera = new T.PerspectiveCamera(40, 1, 0.1, 30);
        if (source) {
          source.updateWorldMatrix(true, false);
          source.getWorldPosition(camera.position);
          source.getWorldQuaternion(camera.quaternion);
          if (source instanceof T.PerspectiveCamera) camera.fov = source.fov;
        } else {
          camera.position.set(0, 1.18, 1.2);
          camera.lookAt(0, 1.3, -0.6);
        }
        camera.updateProjectionMatrix();
        RectAreaLightUniformsLib.init();
        for (const f of fixtures.fixtures) {
          const light = new T.RectAreaLight(
            new T.Color(...(f.color as [number, number, number])),
            f.powerWatts / (Math.PI * f.sizeMeters * f.sizeMeters) * 0.35,
            f.sizeMeters,
            f.sizeMeters,
          );
          light.position.set(f.location[0], f.location[2], -f.location[1]);
          light.quaternion.fromArray(f.rotation as [number,number,number,number]);
          scene.add(light);
        }
        scene.add(new T.HemisphereLight(0xe4eeff, 0x8b8575, 0.65));
        const windowLight = new T.DirectionalLight(0xeaf1ff, 0.35);
        const windowFixture = fixtures.fixtures[0];
        windowLight.position.set(
          windowFixture.location[0],
          windowFixture.location[2],
          -windowFixture.location[1],
        );
        windowLight.target.position.set(0, 0.8, -0.65);
        windowLight.castShadow = true;
        windowLight.shadow.mapSize.set(2048, 2048);
        Object.assign(windowLight.shadow.camera, {
          left: -2.5,
          right: 2.5,
          top: 2.5,
          bottom: -2.5,
          near: 0.1,
          far: 10,
        });
        windowLight.shadow.normalBias = 0.008;
        scene.add(windowLight, windowLight.target);
        // MSAA resolves one scene render. The previous accumulation pass rendered the entire
        // animated, shadowed scene twice per frame, and restarted accumulation on each blink.
        const target=new T.WebGLRenderTarget(1,1,{type:T.HalfFloatType,samples:Math.min(4,renderer.capabilities.maxSamples)});
        composer=new EffectComposer(renderer,target);
        composer.addPass(new RenderPass(scene,camera));composer.addPass(new OutputPass());
        mixer = new T.AnimationMixer(model);
        if (gltf.animations[0]) {
          action = mixer.clipAction(gltf.animations[0]);
          action.setLoop(T.LoopOnce, 1);
          action.clampWhenFinished = true;
          action.play();
          action.paused = true;
          mixer.setTime(0);
        }
        const basePosition=camera.position.clone(),baseQuaternion=camera.quaternion.clone();
        const examiner=createExaminerAnimation(model);
        const viewPosition=basePosition.clone(),viewQuaternion=baseQuaternion.clone();
        let yaw=0,pitch=0,looking=false,writing=false;
        let handoffStarted=false,readySent=false;
        const card=model.getObjectByName("P2_TopicCard");
        const paper=model.getObjectByName("P2_BlankPaper");
        if(!card||!paper)throw Error("场景缺少话题卡或草稿纸。");
        model.traverse(o=>{if(o.name.startsWith("P2_CardText"))o.visible=false;});
        const cardInk=createPaperSurface(card,.148,.208),noteInk=createPaperSurface(paper,.146,.208);
        let oldTopic="",oldNotes:PaperNotes|undefined;
        const setFocusView=(target:"room"|"card"|"notes")=>{
          focusRef.current=target;setFocus(target);el.dataset.focus=target;
          if(target!=="notes")textInput.current?.blur();
          else requestAnimationFrame(()=>textInput.current?.focus({preventScroll:true}));
        };
        controls.current={lookUp:()=>{setFocusView("room");yaw=0;pitch=0;el.focus();}};
        const ray=new T.Raycaster(),pointer=new T.Vector2();
        const intersect=(event:PointerEvent)=>{
          const r=el.getBoundingClientRect();pointer.set((event.clientX-r.left)/r.width*2-1,-(event.clientY-r.top)/r.height*2+1);
          ray.setFromCamera(pointer,camera);
          return ray.intersectObjects([cardInk.surface,noteInk.surface],false)[0];
        };
        const write=(event:PointerEvent,start=false)=>{
          const h=intersect(event);if(h?.object!==noteInk.surface||!h.uv)return;
          const point=clampPaperPoint(h.uv.x,1-h.uv.y);if(!point)return;
          const n=live.current.notes;if(n.strokes.length>=20000)return;
          live.current.onNotes({...n,strokes:[...n.strokes,{...point,start}]});
        };
        const down=(e:PointerEvent)=>{
          if(!live.current.interactive)return;
          if(e.button===2){looking=true;setFocusView("room");el.setPointerCapture(e.pointerId);return;}
          if(e.button!==0)return;
          const h=intersect(e);
          if(h?.object===noteInk.surface&&live.current.flow?.handoffReady){
            writing=focusRef.current==="notes";setFocusView("notes");if(writing){el.setPointerCapture(e.pointerId);write(e,true);}
          }else if(h?.object===cardInk.surface&&live.current.flow?.handoffReady)setFocusView("card");
          else el.focus();
        };
        const move=(e:PointerEvent)=>{
          if(!live.current.interactive){looking=false;writing=false;return;}
          if(looking){yaw=T.MathUtils.clamp(yaw-e.movementX*.003,-1.15,1.15);pitch=T.MathUtils.clamp(pitch-e.movementY*.003,-1.0,.55);}
          if(writing)write(e);
          if(!looking&&!writing){const h=intersect(e);el.style.cursor=h&&live.current.flow?.handoffReady?"pointer":"default";}
        };
        const up=(e:PointerEvent)=>{looking=false;writing=false;if(el.hasPointerCapture(e.pointerId))el.releasePointerCapture(e.pointerId);};
        const key=(e:KeyboardEvent)=>{
          if(e.key==="Escape"&&focusRef.current!=="room"){e.preventDefault();e.stopPropagation();controls.current?.lookUp();}
          if(e.key.toLowerCase()==="e"&&e.target===el&&live.current.flow?.handoffReady){e.preventDefault();setFocusView(focusRef.current==="room"?"card":"room");}
        };
        const context=(e:Event)=>e.preventDefault();
        el.addEventListener("pointerdown",down);el.addEventListener("pointermove",move);el.addEventListener("pointerup",up);el.addEventListener("pointercancel",up);el.addEventListener("contextmenu",context);
        const section=el.parentElement!;section.addEventListener("keydown",key);
        cleanup.push(()=>{el.removeEventListener("pointerdown",down);el.removeEventListener("pointermove",move);el.removeEventListener("pointerup",up);el.removeEventListener("pointercancel",up);el.removeEventListener("contextmenu",context);section.removeEventListener("keydown",key);});
        observer = new ResizeObserver(() => {
          if (!renderer) return;
          renderer.setSize(el.clientWidth, el.clientHeight);
          composer!.setSize(el.clientWidth,el.clientHeight);
          camera.aspect = el.clientWidth / el.clientHeight;
          camera.updateProjectionMatrix();
        });
        observer.observe(el);
        const lost=(event:Event)=>{event.preventDefault();setError("三维画面暂时中断，考试记录仍保留。点击重建画面恢复。");};
        renderer.domElement.addEventListener("webglcontextlost",lost);cleanup.push(()=>renderer?.domElement.removeEventListener("webglcontextlost",lost));
        last = performance.now();
        const paperPosition=new T.Vector3(),lookOffset=new T.Vector3(0,.38,.27),lookCamera=new T.PerspectiveCamera(),lookRotation=new T.Quaternion(),lookEuler=new T.Euler(0,0,0,"YXZ");
        const render = (now: number) => {
          if (cancel) return;
          if(!shouldDraw(now,last,!document.hidden,live.current.interactive,true)){frame=requestAnimationFrame(render);return;}
          const delta = Math.min((now - last) / 1000, 0.1);
          last = now;
          const current=live.current;
          examiner.restore();
          const handed=current.flow?.handoffReady===true;
          const wantsHandoff=current.flow?.stage==="instructions";
          if(handed&&!handoffStarted){
            handoffStarted=true;readySent=true;
            if(action){action.paused=false;mixer!.setTime(14);action.paused=true;}
          }else if(wantsHandoff&&!handoffStarted){
            handoffStarted=true;animating=true;
            if(action){action.reset();action.paused=false;action.play();}
          }
          if(animating){
            mixer!.update(delta);
            if(action&&action.time>=12&&!readySent){readySent=true;current.onPropsReady();}
            if(action&&action.time>=13.95){animating=false;action.paused=true;}
          }
          examiner.update(delta,{stage:current.flow?.stage??"greeting",mouth:mouthLevel.current,articulation:articulation?.current,handoff:animating||wantsHandoff});
          const content=(wantsHandoff||handed)?current.topic:"";
          if(content!==oldTopic||!oldNotes){cardInk.draw(content,null);oldTopic=content;}
          if(oldNotes!==current.notes){noteInk.draw("",current.notes);oldNotes=current.notes;}
          if(focusRef.current!=="room"){
            const object=focusRef.current==="card"?card:paper;
            object.getWorldPosition(paperPosition);
            viewPosition.copy(paperPosition).add(lookOffset);
            lookCamera.position.copy(viewPosition);lookCamera.lookAt(paperPosition);viewQuaternion.copy(lookCamera.quaternion);
          }else{
            viewPosition.copy(basePosition);
            lookEuler.set(pitch,yaw,0,"YXZ");viewQuaternion.copy(baseQuaternion).multiply(lookRotation.setFromEuler(lookEuler));
          }
          const ease=1-Math.exp(-delta*9);
          camera.position.lerp(viewPosition,ease);camera.quaternion.slerp(viewQuaternion,ease);
          composer!.render(delta);
          frame = requestAnimationFrame(render);
        };
        setStatus("正在预热场景着色器…");
        await renderer.compileAsync(scene,camera);
        if(cancel)return;
        frame = requestAnimationFrame(render);
        setStatus("按住右键环顾 · 话题卡递到后点击阅读 · 点击草稿纸输入或拖动书写 · Esc 抬头/菜单");
        live.current.onReady();
      } catch (e) {
        if (!cancel) {
          setError(errorText(e));
          setStatus("场景载入失败，可检查设置中的 GLB 路径。");
        }
      }
    })();
    return () => {
      cancel = true;
      controls.current = null;
      cancelAnimationFrame(frame);
      observer?.disconnect();
      cleanup.forEach((fn) => fn());
      mixer?.stopAllAction();
      if (model) {
        mixer?.uncacheRoot(model);
        releaseScene?.();
      }
      renderer?.dispose();
      composer?.passes.forEach(pass=>pass.dispose());composer?.dispose();
      renderer?.forceContextLoss();
      renderer?.domElement.remove();
    };
  }, [port, renderScale,assetKey,reload]);
  return (
    <section className="room-panel speaking-room">
      <div ref={host} className="room-canvas" tabIndex={interactive?0:-1} role="region" aria-label="三维口语考场" data-focus={focus}/>
      <textarea ref={textInput} className="paper-keyboard-input" aria-label="草稿纸键盘输入" wrap="off"
        value={notes.text} maxLength={1500} tabIndex={-1}
        onChange={e=>onNotes({...live.current.notes,text:e.target.value})}
        onKeyDown={e=>{if(e.key==="Escape"){e.preventDefault();e.stopPropagation();controls.current?.lookUp();}}}/>
      <div className="room-controls"><span>{error||status}</span></div>
      {error&&<p role="alert" className="game-error">{error}<button onClick={()=>setReload(x=>x+1)}>重建三维画面</button></p>}
    </section>
  );
}
