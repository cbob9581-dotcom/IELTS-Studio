import { useEffect, useState } from "react";
import {TaskStatus} from "./cloud/TaskStatus";
import {TaskCenter} from './tasks/TaskCenter';
import {SourceLibrary} from "./source/SourceLibrary";
import { Home } from "./pages/Home";
import { version } from "../package.json";
import {ServiceStatus} from "./services/ServiceStatus";
import {
  LayoutDashboard,
  Library,
  Clock3,
  Settings,
  ArrowLeft,
} from "lucide-react";
import { Library as PaperLibrary } from "./pages/Library";
import { Settings as SettingsPage } from "./pages/Settings";
import { History } from "./pages/History";
import { Workspace } from "./exam/Workspace";
import { ReportView } from "./exam/ReportView";
import { gradeObjective } from "./domain/grading";
import {
  getDesktop,
  errorText,
  type DesktopPort,
  type StorageRoots,
} from "./platform/desktop";
import type { PaperVersion, Mode, Section } from "./domain/paper";
import {
  createAttempt,
  createNextExamAttempt,
  selectScope,
  restoreClock,
  type AttemptSnapshot,
  type Scope,
} from "./domain/session";
export default function App() {
  const [page, setPage] = useState("home");
  const [port, setPort] = useState<DesktopPort | null>(null);
  const [roots, setRoots] = useState<StorageRoots | null>(null);
  const [error, setError] = useState("");
  const [sourceProjectId,setSourceProjectId]=useState<string>();
  const [cloudJobId,setCloudJobId]=useState<string>();
  const [mode, setMode] = useState<Mode>("training");
  const [working, setWorking] = useState<{
    paper: PaperVersion;
    attempt: AttemptSnapshot;
  } | null>(null);
  useEffect(() => {
    let mounted = true;
    getDesktop()
      .then(async (p) => {
        if (mounted) setPort(p);
        const r = await p.getRoots();
        if (mounted) setRoots(r);
      })
      .catch((e) => {
        if (mounted) setError(errorText(e));
      });
    return () => {
      mounted = false;
    };
  }, []);
  async function start(
    paper: PaperVersion,
    section: Section,
    scope: Scope,
    targetMode: Mode = mode,
  ) {
    if (!port) throw Error("桌面存储未连接");
    const attempt = await port.saveAttempt(
      createAttempt(paper, targetMode, section, scope),
      0,
    );
    setWorking({ paper, attempt });
    setPage("workspace");
  }
  async function resume(a: AttemptSnapshot) {
    if (!port) return;
    const paper = await port.loadPaper(a.paperId, a.paperVersion);
    let attempt = a;
    if (a.status !== "submitted" && a.status !== "room") {
      attempt = {
        ...a,
        status: "answering",
        clock: restoreClock(a.clock, Date.now()),
        interruptionEvents: [
          ...a.interruptionEvents,
          { at: Date.now(), reason: "从保存的记录恢复" },
        ],
      };
      attempt = await port.saveAttempt(attempt, a.revision);
    }
    setMode(a.mode);
    setWorking({ paper, attempt });
    setPage(a.status === "submitted" ? "report" : "workspace");
  }
  async function advanceExam(submitted:AttemptSnapshot){
    if(!port||!working)return;const next=createNextExamAttempt(working.paper,submitted);if(!next)return;
    const existing=(await port.listAttempts()).find(a=>a.examRunId===next.examRunId&&a.section===next.section);
    const attempt=existing??await port.saveAttempt(next,0);setWorking({paper:working.paper,attempt});setPage(attempt.status==="submitted"?"report":"workspace");
  }
  const workspace = page === "workspace" && working && port;
  if (workspace && working && port) return (
    <Workspace key={working.attempt.id} paper={working.paper}
      initial={working.attempt} port={port}
      onExit={() => { setWorking(null); setPage("home"); }}
      onAdvance={advanceExam}
      onRepeat={() => start(working.paper, working.attempt.section,
        working.attempt.scope, working.attempt.mode).catch(e => setError(errorText(e)))} />
  );
  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <span className="brand-symbol">i.</span>
          <div>
            IELTS<span>STUDIO</span>
          </div>
        </div>
        <div className="sidebar-label">学习空间</div>
        <nav>
          {[
            ["home", "学习首页", LayoutDashboard],
            ["library", "我的题库", Library],
            ["sources", "真题资料库", Library],
            ["tasks", "后台任务", Clock3],
            ["history", "学习记录", Clock3],
            ["settings", "设置", Settings],
          ].map(([key, label, Icon]) => {
            const I = Icon as typeof LayoutDashboard;
            return (
              <button
                disabled={!!workspace}
                className={page === key ? "active" : ""}
                onClick={() => setPage(key as string)}
                key={key as string}
              >
                <I size={18} />
                {label as string}
              </button>
            );
          })}
        </nav>
        <div className="sidebar-bottom">
          <span className="status-dot" />
          本地桌面 · 首版开发中<small>ACADEMIC / {version}</small>
        </div>
      </aside>
      <main>
        <header>
          <span>
            学习空间 <span className="slash">/</span>{" "}
            {page === "home" ? "首页" : workspace ? "考场" : "工作台"}
          </span>
          <span className="local-badge">LOCAL FIRST</span>
        </header>
        <div className={`page ${workspace ? "workspace-page" : ""}`}>
          {port&&<TaskStatus port={port} onOpen={()=>setPage("tasks")}/>}
          {error && (
            <p className="error" role="alert">
              {error}
            </p>
          )}
          {port && !roots && page !== "settings" && (
            <div className="setup-banner">
              首次使用，请先选择资产库与学习记录的保存位置。
              <button onClick={() => setPage("settings")}>设置保存位置</button>
            </div>
          )}
          {page === "home" ? (
            <Home
              onMode={(m) => {
                setMode(m);
                setPage(m);
              }}
            />
          ) : (
            <>
              <button className="back" onClick={() => setPage("home")}>
                <ArrowLeft size={16} /> 返回首页
              </button>
              {page==='tasks'&&port&&<TaskCenter port={port} onOpen={async t=>{try{if(t.kind==='source'){setSourceProjectId(t.owner);setPage('sources');}else{const job=await port.cloudRead(t.owner);if(job.kind==='assessment'&&typeof job.request.attemptId==='string'){await resume(await port.loadAttempt(job.request.attemptId));}else{setCloudJobId(t.owner);setPage('training');}}}catch(e){setError(errorText(e));}}}/>}
              {page === "report" && working ? (
                <ReportView port={port} paper={working.paper} attempt={working.attempt}
                  result={gradeObjective(selectScope(working.paper.sections.find(s=>s.section===working.attempt.section)!,working.attempt.scope),working.attempt.responses,working.attempt.revealedQuestions)} />
              ) : page === "settings" ? (
                <SettingsPage
                  port={port}
                  roots={roots}
                  onRoots={(r) => {
                    setRoots(r);
                    setError("");
                  }}
                />
              ) : page === "sources" ? (
                <SourceLibrary port={roots?port:null} initialProjectId={sourceProjectId}/>
              ) : page === "history" ? (
                <History port={roots ? port : null} onResume={resume} />
              ) : (
                <PaperLibrary
                  initialJobId={cloudJobId}
                  mode={page === "exam" ? "exam" : "training"}
                  port={roots ? port : null}
                  onStart={(p, s, scope) =>
                    start(p, s, scope, page === "exam" ? "exam" : "training")
                  }
                  onSettings={() => setPage("settings")}
                  onSources={(id)=>{setSourceProjectId(id);setPage("sources");}}
                />
              )}
            </>
          )}
          <div className="cloud-status">
            <span className="status-dot amber" />
            云服务配置与接入状态
            {port&&<ServiceStatus port={port} refreshKey={page}/>}
          </div>
        </div>
      </main>
    </div>
  );
}
