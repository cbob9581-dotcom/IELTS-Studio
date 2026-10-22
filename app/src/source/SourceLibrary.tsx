import {sourceHealth} from "./health";
import {ListeningAudioPanel} from "./ListeningAudioPanel";
import { useEffect, useState, useSyncExternalStore, useRef } from "react";
import type { DesktopPort } from "../platform/desktop";
import { errorText } from "../platform/desktop";
import type { Section } from "../domain/paper";
import type { SourceProject, SourceRole, Block, Unit, Entry } from "./types";
import { roleNames, sectionNames } from "./types";
import { sourceRunner } from "./runner";
import { SourcePreview } from "./SourcePreview";
const stateNames: Record<string, string> = {
  pending: "待处理",
  repairing: "正在向原模型反馈原因并修复（最多1次）",
  parsing: "云端解析中",
  scanning: "识别书籍结构",
  ready: "结构识别完成",
  complete: "提取完成",
  media: "待绑定音频",
  failed: "需处理",
};
export function SourceLibrary({ port,initialProjectId }: { port: DesktopPort | null;initialProjectId?:string }) {
  const [items, setItems] = useState<{ id: string; title: string }[]>([]),
    [project, setProject] = useState<SourceProject | null>(null),
    [title, setTitle] = useState("");
  const [roles, setRoles] = useState<SourceRole[]>(["mixed"]),
    [error, setError] = useState(""),
    [saving, setSaving] = useState(false),
    [docId, setDocId] = useState(""),
    [blocks, setBlocks] = useState<Block[]>([]),
    [blockIndex, setBlockIndex] = useState(0);
  const [unit, setUnit] = useState<Partial<Unit>>({
      test: "Test 1",
      section: "reading",
      part: 1,
      role: "questions",
      start: 0,
      end: 0,
    }),
    [editId, setEditId] = useState("");
  const [test, setTest] = useState("Test 1"),
    [selected, setSelected] = useState<string[]>([]),
    [filter, setFilter] = useState("all");
  const pendingLocate = useRef<string[]>([]);
  const runner = port ? sourceRunner(port) : null;
  const progress = useSyncExternalStore(
    runner?.subscribe ?? (() => () => {}),
    runner?.snapshot ?? (() => null),
  );
  const busy = saving || !!progress?.busy;
  useEffect(() => {
    if (port?.sourceAction)
      port
        .sourceAction("list", {})
        .then(setItems)
        .catch((e) => setError(errorText(e)));
  }, [port]);
  useEffect(()=>{if(initialProjectId&&port?.sourceAction){let active=true;port.sourceAction("read",{id:initialProjectId}).then(p=>{if(active){setProject(p);setDocId(p.documents[0]?.id??"");}}).catch(e=>{if(active)setError(errorText(e));});return()=>{active=false;};}},[port,initialProjectId]);
  useEffect(() => {
    if (progress?.project && (!project || project.id === progress.project.id)) {
      setProject(progress.project);

    }
    if (progress?.error) setError(progress.error);
  }, [progress]);
  useEffect(() => {
    setBlocks([]);
    setBlockIndex(0);
    if (project && docId && port?.sourceAction) {
      let active = true;
      port
        .sourceAction("blocks", { id: project.id, targetId: docId })
        .then((b) => {
          if (active) {
            setBlocks(b);
            const n = (b as Block[]).findIndex((x) =>
              pendingLocate.current.includes(x.id),
            );
            if (n >= 0) setBlockIndex(n);
            pendingLocate.current = [];
          }
        })
        .catch(() => {});
      return () => {
        active = false;
      };
    }
  }, [
    port,
    project?.id,
    docId,
    project?.documents.find((d) => d.id === docId)?.blockCount,
  ]);
  async function action(a: string, payload: Record<string, unknown> = {}) {
    if (!port?.sourceAction) throw Error("请从新版桌面程序打开资料库");
    setSaving(true);
    setError("");
    try {
      const p = await port.sourceAction(a, {
        id: project?.id,
        revision: project?.revision,
        ...payload,
      });
      setProject(p);

      return p as SourceProject;
    } finally {
      setSaving(false);
    }
  }
  function perform(f: () => Promise<unknown>) {
    void f().catch((e) => setError(errorText(e)));
  }
  async function run(targets: string[]) {
    if (!runner || !project) return;
    setError("");
    await runner.run(project, targets);
  }
  async function addFiles() {
    if (!port?.chooseFiles) return;
    const files = await port.chooseFiles([
      "pdf",
      "png",
      "jpg",
      "jpeg",
      "webp",
      "txt",
      "md",
      "wav",
      "mp3",
      "ogg",
    ]);
    if (!files.length) return;
    await action("add_documents", {
      files: files.map((path) => ({ path, roles })),
    });
  }
  const document = project?.documents.find((d) => d.id === docId),
    tests = [...new Set(project?.units.map((u) => u.test) ?? [])];
  const questions =
    project?.units.flatMap((u) => u.result?.section?.questions ?? []) ?? [];
  const pendingEntries =
    project?.entries.filter((e) =>
      ["unassigned", "conflict", "source_review"].includes(e.state),
    ) ?? [];
  function locate(entry: Entry) {
    pendingLocate.current = entry.anchor.blockIds;
    setDocId(entry.anchor.documentId);
    if (entry.anchor.documentId === docId) {
      const n = blocks.findIndex((b) => entry.anchor.blockIds.includes(b.id));
      if (n >= 0) setBlockIndex(n);
    }
  }
  return (
    <div className="source-library">
      <div className="eyebrow">SOURCE LIBRARY · ACADEMIC</div>
      <h1>真题资料库</h1>
      <p className="muted">
        整本题册分 Test
        整理。答案、解析、范文和音频可随时补充；已发布的考试版本保持独立。
      </p>
      {!port?.sourceAction && (
        <p className="notice">资料处理需要新版桌面应用，并先设置保存位置。</p>
      )}
      <div className="library-actions">
        <label>
          资料项目名称
          <input
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            maxLength={200}
          />
        </label>
        <button
          disabled={busy || !title.trim() || !port?.sourceAction}
          onClick={() =>
            perform(async () => {
              const p = await action("create", { title });
              setItems((a) => [{ id: p.id, title: p.title }, ...a]);
              setTitle("");
            })
          }
        >
          新建资料项目
        </button>
      </div>
      <div className="library-actions">
        {items.map((i) => (
          <button
            disabled={busy}
            key={i.id}
            aria-pressed={project?.id === i.id}
            onClick={() => perform(() => action("read", { id: i.id }))}
          >
            {i.title}
          </button>
        ))}
      </div>
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}
      {project && (
        <>
          <h2>{project.title}</h2>
          <section className="settings-card"><h3>按 Test 查看处理结果</h3><p className="muted">已提取的部分持续保留。选择需要使用的 Test，由系统整理题目和匹配答案；已完成结果会保留。</p>{sourceHealth(project).map(h=><article className="settings-card" key={h.test}><h4>{h.test} · 已提取 {h.questions} 个题目对象</h4><p>{h.pending} 个单元待处理，{h.failures.length} 个单元需处理，{h.missingAnswers.length} 题缺有来源的答案。</p>{!!h.missingAnswers.length&&<details><summary>查看缺失答案</summary><p>{h.missingAnswers.join('、')}</p></details>}{h.failures.map(f=><p key={f.id} className="error">{f.title}：{f.error}</p>)}<button disabled={busy} onClick={()=>perform(async()=>{const p=await action('confirm_test',{test:h.test});const rank:Record<string,number>={questions:0,answers:1,explanations:2,essays:3,transcript:4};const targets=p.units.filter((u:Unit)=>u.test===h.test&&u.state!=='complete').sort((a:Unit,b:Unit)=>(rank[a.role]??9)-(rank[b.role]??9)||a.part-b.part).map((u:Unit)=>u.id);if(targets.length)await runner?.run(p,targets);})}>整理 {h.test}（文字服务按预算计费）</button></article>)}</section>
          <section className="settings-card">
            <h3>补充题册 / 答案 / 解析 / 音频</h3>
            <p>资料用途可多选。同一文件再次加入会合并用途，原件按内容去重。</p>
            <div className="library-actions">
              {(Object.keys(roleNames) as SourceRole[]).map((r) => (
                <label key={r} className="check-row">
                  <input
                    type="checkbox"
                    checked={roles.includes(r)}
                    onChange={(e) =>
                      setRoles((v) =>
                        e.target.checked ? [...v, r] : v.filter((x) => x !== r),
                      )
                    }
                  />
                  {roleNames[r]}
                </label>
              ))}
            </div>
            <button
              disabled={busy || !roles.length || !port?.chooseFiles}
              onClick={() => perform(addFiles)}
            >
              选择文件并加入资料项目
            </button>
            <p className="muted">
              点击处理会将资料发送到已配置的云服务，可能计费。原件先保存在本地资产库；题组失败可单独重试。单文件上限
              200MB；PDF 会按页数和体积自动分批，页码仍对应原书。
            </p>
          </section>
          {progress?.busy && (
            <div className="notice" role="status">
              正在处理：
              {project.documents.find((d) => d.id === progress.targetId)
                ?.name ??
                project.units.find((u) => u.id === progress.targetId)?.title ?? project.audioDrafts?.find(d=>d.id===progress.targetId)?.test}
              。可以切换页面；关闭应用后需手动继续。
              <button onClick={() => runner?.pause()}>
                {progress.paused ? "当前请求结束后暂停" : "停止后续处理"}
              </button>
            </div>
          )}
          <div className="source-summary">
            题目 {questions.length} · 缺答案{" "}
            {
              questions.filter(
                (q) =>
                  !["writing_task", "speaking_prompt"].includes(q.kind) &&
                  !q.answer?.accepted?.length,
              ).length
            }{" "}
            · 冲突 {questions.filter((q) => q.conflicts?.length).length} ·
            待匹配 {pendingEntries.length}
          </div>
          <section className="settings-card">
            <h3>源文件</h3>
            <div className="library-actions">
              <button
                disabled={
                  busy ||
                  !project.documents.some(
                    (d) => !["ready", "media"].includes(d.state),
                  )
                }
                onClick={() =>
                  perform(() =>
                    run(
                      project.documents
                        .filter((d) => !["ready", "media"].includes(d.state))
                        .map((d) => d.id),
                    ),
                  )
                }
              >
                处理未完成文件
              </button>
              <button
                disabled={busy}
                onClick={() => perform(() => action("read"))}
              >
                刷新
              </button>
            </div>
            {project.documents.filter(d=>d.state!=="media").map((d) => (
              <article className="source-document" key={d.id}>
                <button onClick={() => setDocId(d.id)}>{d.name}</button>
                <span>
                  {d.roles.map((r) => roleNames[r]).join(" / ")} ·{" "}
                  {stateNames[d.state] ?? d.state}
                </span>
                {d.pageCount > 0 && <span>{d.pageCount} 页</span>}
                  {!!d.unassignedRanges?.length&&<span>非试题 / 待分配范围 {d.unassignedRanges.length} 段（原文保留）</span>}
                {d.blockCount > 0 && (
                  <span>
                    结构扫描 {d.scanCursor}/{d.blockCount} 块
                  </span>
                )}
                {d.blockCount > 0 && <button disabled={busy} onClick={()=>perform(()=>action("reindex",{targetId:d.id}))}>按原书标题重新分组（保留已确认题组）</button>}
                {d.operation?.label && (
                  <span>
                    {d.operation.label}
                    {d.operation.total
                      ? ` ${d.operation.done ?? 0}/${d.operation.total}`
                      : "，等待云端更新"}
                  </span>
                )}
                {d.error && (
                  <p className="error">
                    {d.error}；已完成内容保留，可仅继续此文件。
                  </p>
                )}
                {!["ready", "media"].includes(d.state) && (
                  <button
                    disabled={busy}
                    onClick={() => perform(() => run([d.id]))}
                  >
                    继续此文件
                  </button>
                )}
                {d.state === "failed" && (
                  <button
                    disabled={busy}
                    onClick={() =>
                      perform(async () => {
                        const next = await action("regenerate_step", {
                          targetId: d.id,
                        });
                        await runner?.run(next, [d.id]);
                      })
                    }
                  >
                    重新请求当前步骤（可能计费）
                  </button>
                )}

              </article>
            ))}
          </section>
          <ListeningAudioPanel port={port!} project={project} busy={busy} action={action} run={(p,ids)=>runner!.run(p,ids)} report={e=>setError(errorText(e))}/>
          <details><summary>高级资料管理（可选）</summary>
          <div className="source-workspace">
            <div>
              {document && (
                <SourcePreview
                  port={port!}
                  projectId={project.id}
                  document={document}
                  block={blocks[blockIndex]}
                />
              )}
              {!!blocks.length && (
                <section className="settings-card">
                  <h3>版面块定位与题组范围</h3>
                  <label>
                    当前版面块
                    <select
                      value={blockIndex}
                      onChange={(e) => setBlockIndex(Number(e.target.value))}
                    >
                      {blocks.map((b, i) => (
                        <option key={b.id} value={i}>
                          {i + 1} · {b.page ? `第 ${b.page} 页` : "文本"} ·{" "}
                          {b.text.slice(0, 60) || b.type}
                        </option>
                      ))}
                    </select>
                  </label>
                  <button
                    disabled={busy}
                    onClick={() => setUnit({ ...unit, start: blockIndex })}
                  >
                    设为题组起点
                  </button>
                  <button
                    disabled={busy}
                    onClick={() => setUnit({ ...unit, end: blockIndex })}
                  >
                    设为题组终点
                  </button>
                  <UnitForm
                    unit={unit}
                    blocks={blocks}
                    disabled={busy}
                    onChange={setUnit}
                  />
                  <button
                    disabled={busy}
                    onClick={() =>
                      perform(async () => {
                        await action("unit_save", {
                          targetId: editId,
                          spec: { ...unit, documentId: docId },
                        });
                        setEditId("");
                      })
                    }
                  >
                    {editId ? "保存题组修改（范围变化需重新提取）" : "新增题组"}
                  </button>
                </section>
              )}
            </div>
            <div>
              <section className="settings-card">
                <h3>Test 与题组</h3>
                <p className="muted">
                  结构识别结果需要核对；同一题组应完整包含题干、选项和图表。可修改
                  Test、用途及范围。
                </p>
                <label>
                  筛选 Test
                  <select
                    value={filter}
                    onChange={(e) => setFilter(e.target.value)}
                  >
                    <option value="all">全部</option>
                    {tests.map((t) => (
                      <option key={t}>{t}</option>
                    ))}
                  </select>
                </label>
                <button
                  disabled={
                    busy ||
                    !project.units.some(
                      (u) => u.confirmed && u.state !== "complete",
                    )
                  }
                  onClick={() =>
                    perform(() =>
                      run(
                        project.units
                          .filter(
                            (u) =>
                              u.confirmed &&
                              u.state !== "complete" &&
                              (filter === "all" || u.test === filter),
                          )
                          .map((u) => u.id),
                      ),
                    )
                  }
                >
                  提取已确认的未完成题组
                </button>
                {project.units
                  .filter((u) => filter === "all" || u.test === filter)
                  .map((u) => (
                    <article className="source-unit" key={u.id}>
                      <h4>
                        {u.test} · {sectionNames[u.section]} · Part {u.part} ·{" "}
                        {roleNames[u.role]}
                      </h4>
                      <p>
                        {u.title} · 块 {u.start + 1}—{u.end + 1} ·{" "}
                        {stateNames[u.state] ?? u.state}
                      </p>
                      <label className="check-row">
                        <input
                          type="checkbox"
                          disabled={busy}
                          checked={u.confirmed}
                          onChange={(e) =>
                            perform(() =>
                              action("confirm_unit", {
                                targetId: u.id,
                                confirmed: e.target.checked,
                              }),
                            )
                          }
                        />
                        已核对 Test、科目、用途和题组范围
                      </label>
                      <button
                        disabled={busy}
                        onClick={() => {
                          setDocId(u.documentId);
                          setUnit({ ...u });
                          setEditId(u.id);
                          setBlockIndex(u.start);
                        }}
                      >
                        查看 / 修改范围
                      </button>
                      <button
                        disabled={
                          busy || !u.confirmed || u.state === "complete"
                        }
                        onClick={() => perform(() => run([u.id]))}
                      >
                        提取 / 重试此题组
                      </button>
                      <button
                        disabled={busy}
                        onClick={() =>
                          perform(() =>
                            action("delete_unit", { targetId: u.id }),
                          )
                        }
                      >
                        移除此草稿题组
                      </button>
                      {u.error && (
                        <p className="error">{u.error}；其他题组不受影响。</p>
                      )}
                      {u.state === "failed" && (
                        <button
                          disabled={busy}
                          onClick={() =>
                            perform(async () => {
                              const next = await action("regenerate_step", {
                                targetId: u.id,
                              });
                              await runner?.run(next, [u.id]);
                            })
                          }
                        >
                          重新请求此题组（可能计费）
                        </button>
                      )}
                      {u.result?.warnings?.map((w, i) => (
                        <p className="notice" key={i}>
                          {w}
                        </p>
                      ))}
                      {u.result?.section?.questions?.map((q) => (
                        <details key={q.id}>
                          <summary>
                            第 {q.number} 题 ·{" "}
                            {q.answer?.accepted?.length
                              ? "已匹配答案"
                              : "待补答案"}
                            {q.conflicts?.length ? " · 答案冲突" : ""}
                          </summary>
                          <p>{q.prompt}</p>
                          <p>
                            答案：{JSON.stringify(q.answer?.accepted ?? [])}
                          </p>
                          {q.conflicts?.map((e) => (
                            <p key={e.id}>冲突候选：{e.quote}</p>
                          ))}
                          <p>解析/范文：{q.reviewMaterials?.length ?? 0} 条</p>
                        </details>
                      ))}
                    </article>
                  ))}
              </section>
            </div>
          </div>
          <section className="settings-card">
            <h3>待确认匹配</h3>
            <p>
              只有
              Test、科目、题号都明确且不冲突时才自动附加。答案和解析保留各自来源。
            </p>
            {pendingEntries.length === 0 ? (
              <p>当前没有待确认条目。</p>
            ) : (
              pendingEntries.map((e) => (
                <EntryReview
                  key={`${e.id}-${e.state}`}
                  entry={e}
                  disabled={busy}
                  onLocate={() => locate(e)}
                  onMatch={(test, section, numbers) =>
                    perform(() =>
                      action("entry_match", {
                        targetId: e.id,
                        test,
                        section,
                        numbers,
                      }),
                    )
                  }
                  onResolve={(useNew) =>
                    perform(() =>
                      action("resolve_conflict", { targetId: e.id, useNew }),
                    )
                  }
                />
              ))
            )}
          </section>
          </details>
          <section className="settings-card">
            <h3>选择已整理的试卷</h3>
            <label>
              选择 Test
              <input
                value={test}
                disabled={busy}
                onChange={(e) => {
                  setTest(e.target.value);
                  setSelected([]);

                }}
                list="source-test-list"
              />
              <datalist id="source-test-list">
                {tests.map((t) => (
                  <option value={t} key={t} />
                ))}
              </datalist>
            </label>
            {project.units
              .filter((u) => u.test === test && u.role === "questions")
              .map((u) => (
                <label key={u.id} className="check-row">
                  <input
                    disabled={busy}
                    type="checkbox"
                    checked={selected.includes(u.id)}
                    onChange={(e) => {
                      setSelected((a) =>
                        e.target.checked
                          ? [...a, u.id]
                          : a.filter((id) => id !== u.id),
                      );

                    }}
                  />
                  {sectionNames[u.section]} Part {u.part} ·{" "}
                  {stateNames[u.state] ?? u.state}
                </label>
              ))}
            <button
              className="primary"
              disabled={busy || !project.units.some(u=>u.test===test&&u.role==="questions"&&u.state==="complete")}
              onClick={() =>
                perform(() =>
                  action("publish", { test, units: selected.length?selected:project.units.filter(u=>u.test===test&&u.role==="questions"&&u.state==="complete").map(u=>u.id) }),
                )
              }
            >
              使用这套试卷
            </button>
            <p className="muted">
              缺少答案、关键图片、音频或存在答案冲突时，发布会被阻止。缺少解析不影响考试。发布后可在“我的题库”选择。
            </p>
            {project.publications.map((v, i) => (
              <p key={i}>
                {v.test} · 已发布版本 {v.version} ·{" "}
                {new Date(v.at).toLocaleString()}
              </p>
            ))}
          </section>
        </>
      )}
    </div>
  );
}
function UnitForm({
  unit: u,
  blocks,
  disabled,
  onChange,
}: {
  unit: Partial<Unit>;
  blocks: Block[];
  disabled: boolean;
  onChange: (u: Partial<Unit>) => void;
}) {
  return (
    <fieldset disabled={disabled}>
      <label>
        Test
        <input
          value={u.test ?? ""}
          onChange={(e) => onChange({ ...u, test: e.target.value })}
        />
      </label>
      <label>
        科目
        <select
          value={u.section}
          onChange={(e) =>
            onChange({ ...u, section: e.target.value as Section })
          }
        >
          {Object.entries(sectionNames).map(([v, n]) => (
            <option value={v} key={v}>
              {n}
            </option>
          ))}
        </select>
      </label>
      <label>
        用途
        <select
          value={u.role}
          onChange={(e) =>
            onChange({ ...u, role: e.target.value as SourceRole })
          }
        >
          {Object.entries(roleNames)
            .filter(([v]) => !["mixed", "audio"].includes(v))
            .map(([v, n]) => (
              <option value={v} key={v}>
                {n}
              </option>
            ))}
        </select>
      </label>
      <label>
        Part / Passage / Task
        <input
          type="number"
          min={1}
          max={4}
          value={u.part ?? 1}
          onChange={(e) => onChange({ ...u, part: Number(e.target.value) })}
        />
      </label>
      <label>
        起始块
        <input
          type="number"
          min={1}
          max={blocks.length}
          value={(u.start ?? 0) + 1}
          onChange={(e) =>
            onChange({ ...u, start: Number(e.target.value) - 1 })
          }
        />
      </label>
      <label>
        结束块
        <input
          type="number"
          min={1}
          max={blocks.length}
          value={(u.end ?? 0) + 1}
          onChange={(e) => onChange({ ...u, end: Number(e.target.value) - 1 })}
        />
      </label>
    </fieldset>
  );
}
function EntryReview({
  entry: e,
  disabled,
  onLocate,
  onMatch,
  onResolve,
}: {
  entry: Entry;
  disabled: boolean;
  onLocate: () => void;
  onMatch: (t: string, s: Section, n: number[]) => void;
  onResolve: (useNew: boolean) => void;
}) {
  const [test, setTest] = useState(e.test),
    [section, setSection] = useState(e.section),
    [numbers, setNumbers] = useState(e.numbers.join(","));
  return (
    <article className="source-unit">
      <h4>
        {roleNames[e.kind]} ·{" "}
        {e.state === "conflict" ? "答案冲突" : "尚未确定对应题目"}
      </h4>
      {e.evidenceKind==='visual_transcription'&&e.sourceVerified!==true&&<p className="notice">此条来自原页图片转录，请先点击定位核对原图，再确认匹配。尚未作为判分答案。</p>}<pre className="source-text">{e.text || e.quote}</pre>
      <p>
        {e.test} · {sectionNames[e.section]} · 题号 {e.numbers.join(", ")} ·
        来源第 {e.anchor?.page ?? "?"} 页
      </p>
      <button onClick={onLocate}>查看原文来源</button>
      {e.state === "conflict" ? (
        <>
          <button disabled={disabled} onClick={() => onResolve(false)}>
            保留已有答案
          </button>
          <button disabled={disabled} onClick={() => onResolve(true)}>
            采用此来源的答案
          </button>
        </>
      ) : (
        <>
          <label>
            匹配 Test
            <input value={test} onChange={(ev) => setTest(ev.target.value)} />
          </label>
          <label>
            匹配科目
            <select
              value={section}
              onChange={(ev) => setSection(ev.target.value as Section)}
            >
              {Object.entries(sectionNames).map(([v, n]) => (
                <option value={v} key={v}>
                  {n}
                </option>
              ))}
            </select>
          </label>
          <label>
            题号（逗号分隔）
            <input
              value={numbers}
              onChange={(ev) => setNumbers(ev.target.value)}
            />
          </label>
          <button
            disabled={disabled || !/^\d+(\s*,\s*\d+)*$/.test(numbers)}
            onClick={() =>
              onMatch(test, section, numbers.split(",").map(Number))
            }
          >
            确认对应关系
          </button>
        </>
      )}
    </article>
  );
}
