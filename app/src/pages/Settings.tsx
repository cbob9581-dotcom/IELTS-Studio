import { useState } from "react";
import type { DesktopPort, StorageRoots } from "../platform/desktop";
import { errorText } from "../platform/desktop";
import { getCapabilities } from "../services/capabilities";
import { ConfigurationPanel } from "../configuration/ConfigurationPanel";
export function Settings({
  port,
  roots,
  onRoots,
}: {
  port: DesktopPort | null;
  roots: StorageRoots | null;
  onRoots: (r: StorageRoots) => void;
}) {
  const [draft, setDraft] = useState<StorageRoots>(
    roots ?? { assets: "", study: "" },
  );
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState("");
  const [review, setReview] = useState(false);
  async function choose(key: keyof StorageRoots) {
    try {
      const path = await port?.chooseDirectory();
      if (path) setDraft((d) => ({ ...d, [key]: path }));
    } catch (e) {
      setError(errorText(e));
    }
  }
  async function save() {
    if (!port) return;
    setReview(false);
    setError("");
    setMessage("");
    setBusy(true);
    try {
      const r = roots
        ? await port.migrateRoots(draft)
        : await port.setRoots(draft);
      onRoots(r);
      setMessage(
        roots
          ? "迁移已完成，原目录保留。"
          : "保存目录已设置，开发验证卷已载入。",
      );
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  }
  return (
    <>
      <div className="eyebrow">YOUR LOCAL WORKSPACE</div>
      <h1>设置</h1>
      <p className="muted">
        资产与学习记录各有自己的位置。更换目录时先复制、校验，再切换。
      </p>
      <section className="settings-card">
        <h2>保存位置</h2>
        {(["assets", "study"] as const).map((key) => (
          <label className="path-row" key={key}>
            <span>{key === "assets" ? "资产库目录" : "学习记录目录"}</span>
            <div>
              <input
                aria-label={key === "assets" ? "资产库目录" : "学习记录目录"}
                value={draft[key]}
                onChange={(e) =>
                  setDraft((d) => ({ ...d, [key]: e.target.value }))
                }
                placeholder="请选择目录"
                disabled={busy}
              />
              <button disabled={!port || busy} onClick={() => choose(key)}>
                浏览…
              </button>
            </div>
          </label>
        ))}
        <p className="muted">
          两个目录不能相同或互相嵌套。迁移目标须为空，旧数据不会自动删除。
        </p>
        <button
          className="primary"
          disabled={
            !port ||
            busy ||
            !draft.assets ||
            !draft.study ||
            (roots?.assets === draft.assets && roots?.study === draft.study)
          }
          onClick={() => (roots ? setReview(true) : save())}
        >
          {busy ? "正在复制与校验…" : roots ? "迁移到新位置" : "保存目录设置"}
        </button>
        {error && (
          <p className="error" role="alert">
            {error}
          </p>
        )}
        {message && <p role="status">{message}</p>}
      </section>
      <section className="settings-card">
        <h2>后台接入进度</h2>
        <p className="muted">
          当前不发送云端请求。下方可以填写并保存配置，实际服务调用将在对应后台接通后启用。
        </p>
        {getCapabilities().map((c) => (
          <div className="service-row" key={c.name}>
            <b>{c.name}</b>
            <span>待接入</span>
          </div>
        ))}
      </section>
      <ConfigurationPanel port={port} />
      {review && (
        <div className="modal-backdrop">
          <section
            role="dialog"
            aria-modal="true"
            aria-labelledby="migration-title"
            className="modal"
          >
            <h2 id="migration-title">确认保存位置迁移</h2>
            <p>复制并校验成功后切换目录。原数据保留。</p>
            <p>资产：{draft.assets}</p>
            <p>学习记录：{draft.study}</p>
            <button onClick={() => setReview(false)}>取消</button>{" "}
            <button className="primary" onClick={save}>
              确认迁移
            </button>
          </section>
        </div>
      )}
    </>
  );
}
