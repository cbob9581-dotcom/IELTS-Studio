import {
  lazy,
  Suspense,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";
import type { PaperVersion } from "../domain/paper";
import type { AttemptSnapshot } from "../domain/session";
import { advanceClock, selectScope } from "../domain/session";
import { gradeObjective } from "../domain/grading";
import type { DesktopPort } from "../platform/desktop";
import { SaveQueue, errorText, confirmAction } from "../platform/desktop";
import { setGameFullscreen } from "../game/fullscreen";
import { defaultConfig } from "../configuration/types";
const RoomView = lazy(() =>
  import("../rooms/RoomView").then((m) => ({ default: m.RoomView })),
);
const SpeakingSession = lazy(() =>
  import("../speaking/SpeakingSession").then(m => ({default:m.SpeakingSession})),
);
import {AmbientNoise} from "./AmbientNoise";
import { ExamView } from "./ExamView";
import { ReportView } from "./ReportView";
export function Workspace({
  paper,
  initial,
  port,
  onExit,
  onRepeat,
  onAdvance,
}: {
  paper: PaperVersion;
  initial: AttemptSnapshot;
  port: DesktopPort;
  onExit: () => void;
  onRepeat: () => void;
  onAdvance?: (submitted:AttemptSnapshot)=>Promise<void>;
}) {
  const beforeExit=useRef<()=>Promise<void>>(async()=>{});
  const registerBeforeExit=useCallback((save:()=>Promise<void>)=>{beforeExit.current=save;return()=>{if(beforeExit.current===save)beforeExit.current=async()=>{};};},[]);
  const [draft, setDraft] = useState(initial);
  useEffect(()=>{void port.taskExamActive?.(draft.status!=="submitted").catch(()=>{});return()=>{void port.taskExamActive?.(false).catch(()=>{});};},[port,draft.status==="submitted"]);
  const [menuOpen, setMenuOpen] = useState(false);
  const [computerOpen, setComputerOpen] = useState(initial.section !== "speaking" && initial.status !== "room" && initial.status !== "submitted");
  const [computerMounted, setComputerMounted] = useState(computerOpen);
  useEffect(() => {
    let alive = true;
    setGameFullscreen(true).catch(e => { if (alive) setError(`全屏未能开启：${errorText(e)}`); });
    return () => { alive = false; setGameFullscreen(false).catch(() => {}); };
  }, []);
  const focusRoom = () => requestAnimationFrame(() => document.querySelector<HTMLElement>(".room-canvas")?.focus());
  const [preferences, setPreferences] = useState(defaultConfig());
  useEffect(() => {
    port
      .getConfig()
      .then(setPreferences)
      .catch((e) => setError(errorText(e)));
  }, [port]);
  const ref = useRef(draft);
  const queue = useRef<SaveQueue | null>(null);
  if (!queue.current)
    queue.current = new SaveQueue(port.saveAttempt, initial.revision);
  const [saveState, setSaveState] = useState("已保存");
  const [error, setError] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [confirmSubmit, setConfirmSubmit] = useState(false);
  useEffect(() => {
    const key = (e: KeyboardEvent) => {
      if (e.key !== "Escape" || e.repeat || e.defaultPrevented) return;
      e.preventDefault();
      if (confirmSubmit) setConfirmSubmit(false);
      else if (computerOpen) { setComputerOpen(false); focusRoom(); }
      else setMenuOpen(x => !x);
    };
    window.addEventListener("keydown", key);
    return () => window.removeEventListener("keydown", key);
  }, [computerOpen, confirmSubmit]);

  const debounce = useRef<ReturnType<typeof setTimeout> | null>(null);
  const paused = draft.status === "interrupted";
  const submittingRef = useRef(false);
  const flush = useCallback(async () => {
    if (debounce.current) clearTimeout(debounce.current);
    setSaveState("保存中…");
    try {
      await queue.current!.flush();
      ref.current = { ...ref.current, revision: queue.current!.revision };
      setDraft(ref.current);
      setSaveState("已保存");
      setError("");
    } catch (e) {
      setSaveState("保存失败");
      setError(errorText(e));
      throw e;
    }
  }, []);
  const update = useCallback(
    (a: AttemptSnapshot) => {
      ref.current = { ...a, revision: queue.current!.revision };
      setDraft(ref.current);
      queue.current!.enqueue(ref.current);
      setSaveState("保存中…");
      if (debounce.current) clearTimeout(debounce.current);
      debounce.current = setTimeout(() => {
        flush().catch(() => {});
      }, 500);
    },
    [flush],
  );
  useEffect(() => {
    const unload = (e: BeforeUnloadEvent) => {
      if (queue.current!.dirty) {
        e.preventDefault();
        e.returnValue = "草稿尚未保存";
      }
    };
    window.addEventListener("beforeunload", unload);
    return () => {
      window.removeEventListener("beforeunload", unload);
      if (debounce.current) clearTimeout(debounce.current);
    };
  }, []);
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let disposed = false;
    if ("__TAURI_INTERNALS__" in window)
      import("@tauri-apps/api/window").then(async ({ getCurrentWindow }) => {
        const win = getCurrentWindow();
        const off = await win.onCloseRequested(async (e) => {
          e.preventDefault();
          try {
            if (ref.current.status !== "submitted") {
              if (
                ref.current.mode === "exam" &&
                !(await confirmAction(
                  "退出会记录考试中断，计时不会暂停。确定关闭？",
                ))
              )
                return;
              await beforeExit.current();await flush();
            }
            await win.destroy();
          } catch {
            /* Keep window open on failed save. */
          }
        });
        if (disposed) off();
        else unlisten = off;
      });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [flush]);
  const submit = useCallback(async () => {
    if (submittingRef.current || ref.current.status === "submitted") return;
    submittingRef.current = true;
    setSubmitting(true);
    setConfirmSubmit(false);
    try {
      await beforeExit.current();await flush();
      const result = await port.submitAttempt(
        ref.current.id,
        queue.current!.revision,
      );
      ref.current = result;
      setDraft(result);
      setSaveState("已保存");
      if(result.mode==="exam"&&result.section!=="speaking"&&onAdvance)await onAdvance(result);
    } catch (e) {
      setError(errorText(e));
      throw e;
    } finally {
      submittingRef.current = false;
      setSubmitting(false);
    }
  }, [flush, port, onAdvance]);
  useEffect(() => {
    let previous = performance.now();
    let wall = Date.now();
    const timer = setInterval(() => {
      const nowPerf = performance.now(),
        now = Date.now();
      const elapsed = Math.max(0, nowPerf - previous, now - wall);
      previous = nowPerf;
      wall = now;
      const a = ref.current;
      if (a.section === "speaking" || a.status !== "answering" || submittingRef.current) return;
      const clock = advanceClock(a.clock, now, elapsed);
      const events =
        elapsed > 10000
          ? [
              ...a.interruptionEvents,
              { at: now, reason: "检测到休眠或执行间隔超过 10 秒" },
            ]
          : a.interruptionEvents;
      update({ ...a, clock, interruptionEvents: events });
      if (clock.remainingMs === 0) submit().catch(() => {});
    }, 1000);
    return () => clearInterval(timer);
  }, [update, submit]);
  async function open() {
    const a = ref.current;
    if (a.section === "speaking" || a.status === "submitted") return;
    setComputerMounted(true); setComputerOpen(true);
    if (a.status !== "room") return;
    const now = Date.now();
    update({
      ...a,
      status: "answering",
      clock: {
        ...a.clock,
        deadlineUtcMs: a.mode === "exam" ? now + a.clock.remainingMs : null,
        lastObservedUtcMs: now,
      },
    });
  }
  async function exit() {
    if (ref.current.status !== "submitted") {
      if (
        ref.current.mode === "exam" &&
        !(await confirmAction(
          "退出会记录中断；考试截止时间继续有效。确定退出？",
        ))
      )
        return;
      if (ref.current.status === "answering")
        update({
          ...ref.current,
          status: "interrupted",
          interruptionEvents: [
            ...ref.current.interruptionEvents,
            { at: Date.now(), reason: "主动离开练习" },
          ],
        });
      try {
        await beforeExit.current();await flush();
      } catch {
        return;
      }
    }
    await setGameFullscreen(false).catch(() => {});
    onExit();
  }
  const noiseRecorded=useCallback((kind:string)=>update({...ref.current,environmentEvents:[...(ref.current.environmentEvents??[]),{at:Date.now(),kind}]}),[update]);
  const seconds = Math.ceil(draft.clock.remainingMs / 1000);
  const time = `${Math.floor(seconds / 60)
    .toString()
    .padStart(2, "0")}:${(seconds % 60).toString().padStart(2, "0")}`;
  return (
    <div className="game-session" data-section={draft.section}>
      <AmbientNoise config={preferences} section={draft.section} seed={draft.id} active={draft.status==="answering"} onEvent={noiseRecorded} sequence={draft.environmentEvents?.length??0}/>
      <div className="game-world" inert={menuOpen || computerOpen || confirmSubmit || draft.status === "submitted"}>
        <Suspense fallback={<p className="game-loading">正在进入三维考场…</p>}>
          {draft.section === "speaking" ? (
            <SpeakingSession registerBeforeExit={registerBeforeExit} port={port} paper={paper} attempt={draft} config={preferences} onChange={update} onComplete={submit} interactive={!menuOpen && draft.status !== "submitted"} />
          ) : (
            <RoomView section={draft.section} onOpenPaper={open}
              interactive={!menuOpen && !computerOpen && draft.status !== "submitted"}
              renderScale={preferences.blender.renderScale} />
          )}
        </Suspense>
      </div>
      <div className="game-hud">
        <button onClick={() => setMenuOpen(true)} aria-label="打开考场菜单">☰ <kbd>Esc</kbd></button>
        <span>{draft.section === "speaking" ? "SPEAKING" : draft.section.toUpperCase()}</span>
        <span role="status">{saveState}</span>
        {draft.section !== "speaking" && draft.status !== "room" && <span className="timer">{draft.status === "submitted" ? "已提交" : time}</span>}
      </div>
      {error && <div className="game-error" role="alert">{error}。草稿保留在当前窗口。
        <button onClick={() => (ref.current.speaking?.stage==="finished"?submit():flush()).catch(() => {})}>重试保存 / 提交</button></div>}
      {computerMounted && draft.section !== "speaking" && draft.status !== "submitted" && (
        <div className="computer-overlay" hidden={!computerOpen} inert={!computerOpen || menuOpen || confirmSubmit}>
          <section className="computer-monitor" role="dialog" aria-label="机考操作界面" aria-modal="true">
            <div className="monitor-bezel"><span>IELTS · ACADEMIC</span><span>{time}</span>
              <button onClick={() => {setComputerOpen(false);focusRoom();}}>收起机考界面</button>
            </div>
            <div className="computer-screen">
              <ExamView paper={paper} attempt={draft} port={port} onChange={update}
                onSubmit={() => setConfirmSubmit(true)} paused={paused || submitting}
                masterVolume={preferences.audio.masterVolume} outputDeviceId={preferences.audio.outputDeviceId}
                onPause={() => update({...ref.current, status: paused ? "answering" : "interrupted",
                  interruptionEvents: paused ? ref.current.interruptionEvents : [...ref.current.interruptionEvents, {at:Date.now(),reason:"训练暂停"}]})} />
            </div>
            <div className="monitor-chin"><span className="monitor-led"/> IELTS STUDIO</div>
          </section>
        </div>
      )}
      {draft.status === "submitted" && draft.section !== "speaking" && <div className="game-report">
        <ReportView port={port} paper={paper} attempt={draft} result={gradeObjective(selectScope(paper.sections.find(s => s.section === draft.section)!,draft.scope),draft.responses,draft.revealedQuestions)} />
        <button onClick={exit}>返回主页</button> {draft.mode==="exam"&&onAdvance?<button onClick={()=>onAdvance(draft).catch(e=>setError(errorText(e)))}>进入下一科</button>:<button onClick={onRepeat}>再次练习（保留本次记录）</button>}
      </div>}
      {draft.status === "submitted" && draft.section === "speaking" && <div className="speaking-status">口语考试已结束，记录已保存。按 Esc 返回主页后可在学习记录中查看。</div>}
      {menuOpen && <div className="game-menu-backdrop"><section className="game-menu" role="dialog" aria-modal="true" aria-label="考场菜单">
        <div className="eyebrow">IELTS STUDIO</div><h2>考场菜单</h2>
        <p>菜单不会暂停考试计时。{saveState}</p>
        <button autoFocus onClick={() => {setMenuOpen(false);focusRoom();}}>继续考试</button>
        <button onClick={() => setGameFullscreen(true).catch(e => setError(errorText(e)))}>恢复全屏</button>
        <button onClick={exit} disabled={submitting}>保存并返回主页</button>
      </section></div>}
      {confirmSubmit && <div className="modal-backdrop"><section role="dialog" aria-modal="true" aria-labelledby="submit-heading" className="modal">
        <h2 id="submit-heading">提交本次作答？</h2><p>作答会先保存，再生成本次记录。提交后不能覆盖，可另开一次练习。</p>
        <button onClick={() => setConfirmSubmit(false)}>继续作答</button>{" "}
        <button className="primary" disabled={submitting} onClick={()=>submit().catch(()=>{})}>确认提交</button>
      </section></div>}
    </div>
  );
}
