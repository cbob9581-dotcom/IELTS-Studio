import { useEffect, useRef, useState } from "react";
import * as T from "three";
import type { Section } from "../domain/paper";
import {shouldDraw} from "./renderBudget";
import { layouts } from "./layouts";
import { shouldOpenPaper } from "./interaction";
export function RoomView({
  section,
  onOpenPaper,
  renderScale = 1,
  interactive = true,
}: {
  section: Section;
  onOpenPaper: () => void;
  renderScale?: number;
  interactive?: boolean;
}) {
  const host = useRef<HTMLDivElement>(null);
  const open = useRef(onOpenPaper);
  open.current = onOpenPaper;
  const enabled = useRef(interactive);
  enabled.current = interactive;
  const [error, setError] = useState("");
  const [ready, setReady] = useState(false);
  useEffect(() => {
    const el = host.current!;
    let renderer: T.WebGLRenderer | undefined;
    let stopped = false;
    let frame = 0;
    let observer: ResizeObserver | undefined;
    const scene = new T.Scene();
    const camera = new T.PerspectiveCamera(56, 1, 0.05, 40);
    const disposables: (() => void)[] = [];
    try {
      const style = layouts[section];
      scene.background = new T.Color(0xdfe5dc);
      scene.fog = new T.Fog(0xdfe5dc, 7, 15);
      camera.position.set(0, 1.25, 1.5);
      camera.lookAt(0, 1.05, -1);
      renderer = new T.WebGLRenderer({
        antialias: true,
        powerPreference: "high-performance",
      });
      renderer.setPixelRatio(Math.min(devicePixelRatio * renderScale, 2));
      renderer.shadowMap.enabled = true;
      renderer.shadowMap.type = T.PCFShadowMap;
      renderer.toneMapping = T.ACESFilmicToneMapping;
      renderer.toneMappingExposure = 1.2;
      el.appendChild(renderer.domElement);
      const mat = (color: number, roughness = 0.8) =>
        new T.MeshStandardMaterial({ color, roughness });
      const wall = mat(style.wall),
        trim = mat(0xefeee4),
        accent = mat(style.accent),
        wood = mat(0xb69c77),
        metal = mat(0x4d5550, 0.45),
        paper = mat(0xfcfbf4);
      const box = (
        w: number,
        h: number,
        d: number,
        m: T.Material,
        x: number,
        y: number,
        z: number,
      ) => {
        const mesh = new T.Mesh(new T.BoxGeometry(w, h, d), m);
        mesh.position.set(x, y, z);
        mesh.castShadow = true;
        mesh.receiveShadow = true;
        scene.add(mesh);
        return mesh;
      };
      box(6, 0.1, 7, mat(style.floor), 0, -0.05, -0.9);
      box(6, 3.3, 0.12, wall, 0, 1.65, -3.2);
      box(0.12, 3.3, 7, wall, -3, 1.65, -0.9);
      box(0.12, 3.3, 7, wall, 3, 1.65, -0.9);
      box(6, 0.12, 7, trim, 0, 3.3, -0.9);
      for (let i = 0; i < 23; i++)
        box(6, 0.002, 0.008, mat(0x796c57), 0, 0.004, -4 + i * 0.31);
      box(6, 0.12, 0.035, trim, 0, 0.07, -3.12);
      box(0.035, 0.12, 7, trim, -2.92, 0.07, -0.9);
      box(2.15, 1.65, 0.05, trim, -1.5, 2.04, -3.09);
      const glass = new T.MeshBasicMaterial({ color: 0xd4e6e1 });
      box(1.98, 1.48, 0.01, glass, -1.5, 2.04, -3.05);
      box(0.055, 1.5, 0.06, trim, -1.5, 2.04, -3);
      box(2, 0.045, 0.06, trim, -1.5, 2.04, -3);
      box(2.28, 0.06, 0.3, trim, -1.5, 1.2, -3);
      for (let i = 0; i < 8; i++)
        box(2.1, 0.018, 0.09, mat(0xc5cfbc), -1.5, 2.8 + i * 0.04, -3.01);
      const sun = new T.DirectionalLight(0xffefcd, 3.4);
      sun.position.set(-2, 3.8, 0.6);
      sun.castShadow = true;
      sun.shadow.mapSize.set(2048, 2048);
      sun.shadow.camera.left = -4;
      sun.shadow.camera.right = 4;
      sun.shadow.camera.top = 4;
      sun.shadow.camera.bottom = -4;
      sun.shadow.normalBias = 0.025;
      scene.add(sun);
      scene.add(new T.HemisphereLight(0xe9f1ff, 0x8d866f, 1.5));
      box(2.35, 0.075, 1.45, wood, 0, 0.76, -0.15);
      for (const x of [-1.03, 1.03])
        for (const z of [-0.73, 0.43])
          box(0.065, 0.73, 0.065, metal, x, 0.365, z);
      box(2.12, 0.1, 0.06, wood, 0, 0.3, -0.73);
      const texCanvas = document.createElement("canvas");
      texCanvas.width = 512;
      texCanvas.height = 720;
      const ctx = texCanvas.getContext("2d")!;
      ctx.fillStyle = "#fffdf5";
      ctx.fillRect(0, 0, 512, 720);
      ctx.fillStyle = "#34473b";
      ctx.font = "bold 42px Georgia";
      ctx.fillText("IELTS Academic", 45, 85);
      ctx.font = "26px sans-serif";
      ctx.fillText(section.toUpperCase(), 45, 135);
      ctx.font = "19px sans-serif";
      ctx.fillText("DEVELOPMENT PRACTICE", 45, 185);
      ctx.strokeStyle = "#cbd1c4";
      for (let i = 0; i < 10; i++) {
        ctx.beginPath();
        ctx.moveTo(45, 245 + i * 34);
        ctx.lineTo(i % 3 === 0 ? 350 : 465, 245 + i * 34);
        ctx.stroke();
      }
      ctx.fillStyle = "#84937f";
      ctx.fillText("Click paper or press E", 45, 660);
      const texture = new T.CanvasTexture(texCanvas);
      texture.colorSpace = T.SRGBColorSpace;
      const paperMaterial = new T.MeshStandardMaterial({
        map: texture,
        roughness: 0.92,
      });
      const sheet = box(0.34, 0.004, 0.48, paperMaterial, -0.02, 0.803, -0.16);
      sheet.rotation.y = -0.09;
      const pen = box(0.014, 0.014, 0.25, accent, 0.28, 0.81, -0.07);
      pen.rotation.y = -0.25;
      box(0.014, 0.014, 0.04, metal, 0.31, 0.81, 0.07);
      box(0.44, 0.008, 0.32, paper, -0.5, 0.803, -0.24).rotation.y = 0.22;
      // Desk lamp and a warm pool of light.
      const cylinder = (
        r1: number,
        r2: number,
        h: number,
        m: T.Material,
        x: number,
        y: number,
        z: number,
      ) => {
        const o = new T.Mesh(new T.CylinderGeometry(r1, r2, h, 24), m);
        o.position.set(x, y, z);
        o.castShadow = true;
        scene.add(o);
        return o;
      };
      cylinder(0.12, 0.14, 0.025, metal, 0.8, 0.81, -0.55);
      cylinder(0.012, 0.012, 0.45, metal, 0.8, 1.03, -0.55);
      cylinder(0.09, 0.15, 0.14, accent, 0.76, 1.3, -0.55);
      const lamp = new T.PointLight(0xffdba0, 1.8, 2);
      lamp.position.set(0.76, 1.21, -0.55);
      scene.add(lamp);
      // A second seat and low cabinet establish the scale of the room.
      box(0.53, 0.09, 0.5, accent, 1.7, 0.48, -1.7);
      box(0.53, 0.56, 0.07, accent, 1.7, 0.8, -1.93);
      for (const x of [1.48, 1.92])
        for (const z of [-1.9, -1.5])
          box(0.035, 0.46, 0.035, metal, x, 0.23, z);
      box(1.3, 0.72, 0.4, wood, 1.85, 0.36, -2.91);
      for (let i = 0; i < 4; i++)
        box(
          0.23,
          0.36,
          0.02,
          mat([0x536951, 0x9b8158, 0x6e7f8a, 0xc1b18c][i]),
          1.4 + i * 0.19,
          0.91,
          -2.83,
        );
      cylinder(0.18, 0.13, 0.33, mat(0xbdb49d), -2.4, 0.18, -2.5);
      const stem = mat(0x566e45);
      for (let i = 0; i < 13; i++) {
        const leaf = new T.Mesh(new T.SphereGeometry(0.15, 12, 8), stem);
        leaf.scale.set(0.42, 1.8, 0.65);
        leaf.position.set(
          -2.4 + Math.sin(i * 2.1) * 0.2,
          0.5 + i * 0.045,
          -2.5 + Math.cos(i * 2.1) * 0.17,
        );
        leaf.rotation.z = Math.sin(i) * 0.6;
        scene.add(leaf);
      }
      if (section === "listening") {
        const earMat = mat(0x253936, 0.5);
        for (const x of [-0.7, -0.39]) {
          const ear = new T.Mesh(
            new T.TorusGeometry(0.075, 0.024, 12, 28),
            earMat,
          );
          ear.rotation.x = -Math.PI / 2;
          ear.position.set(x, 0.85, 0.2);
          scene.add(ear);
        }
        const band = new T.Mesh(
          new T.TorusGeometry(0.16, 0.012, 12, 30, Math.PI),
          earMat,
        );
        band.rotation.x = -Math.PI / 2;
        band.position.set(-0.55, 0.845, 0.2);
        scene.add(band);
      }
      if (section === "reading") {
        for (let i = 0; i < 3; i++)
          box(
            0.35,
            0.035,
            0.25,
            mat([0x66775c, 0xc0ac87, 0xe5dfcc][i]),
            -0.78,
            0.83 + i * 0.035,
            -0.51,
          ).rotation.y = 0.07 * i;
      }
      if (section === "writing") {
        box(0.12, 0.14, 0.12, accent, -0.82, 0.88, -0.56);
        for (let i = 0; i < 4; i++) {
          const o = box(
            0.007,
            0.22,
            0.007,
            metal,
            -0.86 + i * 0.024,
            1.05,
            -0.56,
          );
          o.rotation.z = (i - 1.5) * 0.08;
        }
      }
      const ray = new T.Raycaster();
      const mouse = new T.Vector2();
      let pointerX = 0,
        pointerY = 0;
      const hit = (event: PointerEvent) => {
        const r = renderer!.domElement.getBoundingClientRect();
        mouse.set(
          ((event.clientX - r.left) / r.width) * 2 - 1,
          (-(event.clientY - r.top) / r.height) * 2 + 1,
        );
        ray.setFromCamera(mouse, camera);
        return ray.intersectObject(sheet).length > 0;
      };
      const move = (event: PointerEvent) => {
        const r = el.getBoundingClientRect();
        pointerX = (event.clientX - r.left) / r.width - 0.5;
        pointerY = (event.clientY - r.top) / r.height - 0.5;
        el.style.cursor = hit(event) ? "pointer" : "default";
      };
      const click = (e: PointerEvent) => {
        if (!enabled.current) return;
        el.focus();
        if (hit(e)) open.current();
      };
      el.addEventListener("pointermove", move);
      el.addEventListener("pointerup", click);
      disposables.push(
        () => el.removeEventListener("pointermove", move),
        () => el.removeEventListener("pointerup", click),
      );
      const lost = (event: Event) => {
        event.preventDefault();
        setError("三维画面暂时不可用，请从菜单退出并重试。");
      };
      renderer.domElement.addEventListener("webglcontextlost", lost);
      disposables.push(() =>
        renderer?.domElement.removeEventListener("webglcontextlost", lost),
      );
      observer = new ResizeObserver(() => {
        const w = el.clientWidth,
          h = el.clientHeight;
        renderer!.setSize(w, h);
        camera.aspect = w / h;
        camera.updateProjectionMatrix();
      });
      observer.observe(el);
      let lastDraw=0;
      const render = (now=performance.now()) => {
        if (stopped) return;
        if(!shouldDraw(now,lastDraw,!document.hidden,enabled.current,false)){frame=requestAnimationFrame(render);return;}lastDraw=now;
        camera.position.x += (pointerX * 0.06 - camera.position.x) * 0.025;
        camera.lookAt(pointerX * 0.12, 1.05 - pointerY * 0.06, -1);
        renderer!.render(scene, camera);
        frame = requestAnimationFrame(render);
      };
      render();
      setReady(true);
    } catch (e) {
      setError("无法初始化三维画面，请从菜单退出并重试。");
      console.warn("Room renderer", e);
    }
    return () => {
      stopped = true;
      cancelAnimationFrame(frame);
      observer?.disconnect();
      disposables.forEach((f) => f());
      scene.traverse((o) => {
        if (o instanceof T.Mesh) {
          o.geometry.dispose();
          for (const m of Array.isArray(o.material)
            ? o.material
            : [o.material]) {
            const map = (m as T.MeshStandardMaterial).map;
            map?.dispose();
            m.dispose();
          }
        }
      });
      renderer?.dispose();
      renderer?.domElement.remove();
    };
  }, [section, renderScale]);
  return (
    <section className="room-panel">
      <div
        ref={host}
        tabIndex={interactive ? 0 : -1}
        role="region"
        aria-label="三维考场"
        className="room-canvas"
        onKeyDown={(e) => {
          if (!enabled.current) return;
          const t = e.target as HTMLElement;
          if (
            shouldOpenPaper({
              key: e.key,
              repeat: e.repeat,
              focused: e.currentTarget === t || e.currentTarget.contains(t),
              editing:
                ["INPUT", "TEXTAREA", "SELECT"].includes(t.tagName) ||
                t.isContentEditable,
            })
          ) {
            e.preventDefault();
            open.current();
          }
        }}
      />
      <div className="room-controls">
        <span>
          {error || (ready ? "点击桌上试卷，或在考场中按 E" : "正在准备考场…")}
        </span>
      </div>
      {section === "speaking" && (
        <p className="notice">
          已有 Blender
          考官、绑定与递卡动作。角色导入和云端语音尚未接通；当前仅可预览提示卡。
        </p>
      )}
    </section>
  );
}
