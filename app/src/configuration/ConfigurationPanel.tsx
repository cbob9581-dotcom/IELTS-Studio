import { useEffect, useState } from "react";
import type { DesktopPort } from "../platform/desktop";
import { errorText } from "../platform/desktop";
import {
  defaultConfig,
  providerIds,
  providerLabels,
  type AppConfig,
  type ProviderId,
  type ProviderConfig,
} from "./types";
const shortLabels: Record<ProviderId, string> = {
  deepseek: "DeepSeek",
  qwenAsr: "Qwen ASR",
  qwenTts: "Qwen TTS",
  mineru: "MinerU",
  evaluation: "口语评测",
};
export function ConfigurationPanel({ port }: { port: DesktopPort | null }) {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [status, setStatus] = useState<Partial<Record<ProviderId, boolean>>>(
    {},
  );
  const [secrets, setSecrets] = useState<Partial<Record<ProviderId, string>>>(
    {},
  );
  const [error, setError] = useState("");
  const [message, setMessage] = useState("");
  const [busy, setBusy] = useState(false);
  const [devices, setDevices] = useState<MediaDeviceInfo[]>([]);
  const [tests,setTests]=useState<Partial<Record<ProviderId,string>>>({});
  async function testConnection(id:ProviderId){
    if(!port||!config)return;setBusy(true);setError("");
    try{await port.saveConfig(config);const r=await port.cloudTest(id);setTests(t=>({...t,[id]:`${r.detail} · ${r.latencyMs} ms · ${new Date(r.testedAt).toLocaleString()}`}));}
    catch(e){setTests(t=>({...t,[id]:`测试失败：${errorText(e)}`}));setError(errorText(e));}
    finally{setBusy(false);}
  }
  function preset(id:ProviderId){
    const values:Record<ProviderId,Partial<ProviderConfig>>={
      deepseek:{baseUrl:"https://api.deepseek.com",model:"deepseek-flash",timeoutSeconds:300,maxTokens:16384,temperature:0.6},
      mineru:{baseUrl:"https://mineru.net/api/v4",model:"vlm",timeoutSeconds:120},
      qwenAsr:{baseUrl:"https://dashscope.aliyuncs.com/compatible-mode/v1",region:"cn-beijing",model:"qwen3-asr-flash",language:"en",timeoutSeconds:120,maxTokens:4096},
      qwenTts:{baseUrl:"https://dashscope.aliyuncs.com/api/v1",region:"cn-beijing",model:"qwen3-tts-flash",voice:"Serena|Elias|Neil",language:"en",timeoutSeconds:120},
      evaluation:{baseUrl:"https://dashscope.aliyuncs.com/compatible-mode/v1",region:"cn-beijing",model:"qwen3.8-omni-flash",timeoutSeconds:300,maxTokens:4096,temperature:0.2}
    };update(id,values[id]);
  }
  useEffect(() => {
    let active = true;
    if (port) {
      Promise.all([port.getConfig(), port.credentialStatus()])
        .then(([c, s]) => {
          if (active) {
            setConfig(c);
            setStatus(s);
          }
        })
        .catch((e) => {
          if (active) setError(errorText(e));
        });
    }
    return () => {
      active = false;
    };
  }, [port]);
  const update = (id: ProviderId, change: Partial<ProviderConfig>) =>
    setConfig((c) =>
      c
        ? {
            ...c,
            providers: {
              ...c.providers,
              [id]: { ...c.providers[id], ...change },
            },
          }
        : c,
    );
  async function save() {
    if (!port || !config) return;
    setBusy(true);
    setMessage("");
    setError("");
    try {
      await port.saveConfig(config);
      setMessage("普通配置已保存到本地。密钥请使用各服务旁的独立保存按钮。");
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  }
  async function secret(id: ProviderId, remove = false) {
    if (!port) return;
    setBusy(true);
    setMessage("");
    setError("");
    try {
      if (remove) await port.clearCredential(id);
      else await port.saveCredential(id, secrets[id] ?? "");
      setSecrets((s) => ({ ...s, [id]: "" }));
      setStatus(await port.credentialStatus());
      setMessage(
        `${shortLabels[id]} 密钥${remove ? "已清除" : "已存入 Windows 凭证管理器"}。`,
      );
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  }
  async function refreshDevices() {
    try {
      if (!navigator.mediaDevices) throw Error("当前环境无法访问音频设备");
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      stream.getTracks().forEach((t) => t.stop());
      setDevices(await navigator.mediaDevices.enumerateDevices());
    } catch (e) {
      setError(errorText(e));
    }
  }
  if (!port)
    return (
      <section className="settings-card">
        <h2>服务与设备配置</h2>
        <p className="muted">请从桌面程序打开，以保存本地设置。</p>
      </section>
    );
  if (!config)
    return (
      <section className="settings-card">
        <h2>服务与设备配置</h2>
        {error ? (
          <>
            <p role="alert" className="error">
              {error}
            </p>
            <button
              onClick={() => {
                setConfig(defaultConfig());
                setError("");
              }}
            >
              使用默认配置重新填写
            </button>
          </>
        ) : (
          <p>正在读取本地配置…</p>
        )}
      </section>
    );
  return (
    <section className="configuration-panel">
      <div className="section-heading">
        <h2>服务与设备配置</h2>
        <span>LOCAL SETTINGS</span>
      </div>
      <p className="muted">
        地址、模型与偏好保存在本地。密钥独立存入 Windows
        凭证管理器，不回显、不写入学习记录。出题、评测、语音和资料解析均通过原生后台请求；真实连通以各服务的测试结果为准。
      </p>
      {providerIds.map((id) => {
        const p = config.providers[id],
          label = shortLabels[id];
        return (
          <details
            className="settings-card provider-card"
            key={id}
            open={id === "deepseek"}
          >
            <summary>
              {providerLabels[id]}{" "}
              <span className="pill">
                {status[id] ? "密钥已保存" : "未配置密钥"} · {p.enabled?"已启用配置":"未启用"}
              </span>
            </summary>
            <button disabled={busy} onClick={()=>preset(id)}>填入参考参数</button>
            <p className="muted">参考参数已包含地址、模型与超时；Qwen 参考区域为北京，其他区域请同步替换地址与密钥。测试使用当前配置并先保存，可能产生少量调用费用。</p>
            {id==="deepseek"&&<p className="notice">出题、整理资料和考官提问使用快速模式，明确关闭 DeepSeek 默认思考过程。建议输出上限 16384、超时 300 秒；输出上限是允许的最大值，不会强制生成这么多内容。</p>}
            {id==="mineru"&&<p className="muted">服务地址填到 https://mineru.net/api/v4。连接探测不能代替实际文件解析；上传任务会复用保存的批次。</p>}
            {id==="evaluation"&&<p className="muted">使用支持音频输入的 Qwen Omni 模型，地址填到 compatible-mode/v1。实际录音参与四项反馈；分数是未校准参考估分。</p>}
            {id==="qwenAsr"&&<p className="muted">当前口语使用非实时 ASR 的 OpenAI 兼容模式 HTTPS 接口，地址填写到 compatible-mode/v1 或完整 chat/completions。实时 WSS 型号尚不支持。</p>}
            {id==="qwenTts"&&<p className="muted">使用 Qwen TTS 非实时 DashScope 接口，地址填到 api/v1 或完整 services/aigc/multimodal-generation/generation。听力双人对话音色填 音色A|音色B|旁白（例如 Cherry|Ethan|Neil），口语考官使用第一个音色。区域须与密钥一致。</p>}
            <div className="config-grid">
              <label className="check-row">
                <input
                  type="checkbox"
                  checked={p.enabled}
                  onChange={(e) => update(id, { enabled: e.target.checked })}
                />
                保存为启用配置
              </label>
              <label>
                服务地址
                <input
                  aria-label={`${label} 服务地址`}
                  placeholder={
                    "https://…"
                  }
                  value={p.baseUrl}
                  onChange={(e) => update(id, { baseUrl: e.target.value })}
                />
              </label>
              {(
                <label>
                  模型
                  <input
                    aria-label={`${label} 模型`}
                    value={p.model}
                    onChange={(e) => update(id, { model: e.target.value })}
                    placeholder="按供应商文档填写模型 ID"
                  />
                </label>
              )}
              <label>
                区域
                <input
                  aria-label={`${label} 区域`}
                  value={p.region}
                  onChange={(e) => update(id, { region: e.target.value })}
                  placeholder="供应商区域（可选）"
                />
              </label>
              {id === "qwenTts" && (
                <label>
                  音色 ID
                  <input
                    aria-label="Qwen TTS 音色"
                    value={p.voice}
                    onChange={(e) => update(id, { voice: e.target.value })}
                  />
                </label>
              )}
              {["qwenAsr", "qwenTts", "evaluation"].includes(id) && (
                <label>
                  语言
                  <input
                    aria-label={`${label} 语言`}
                    value={p.language}
                    onChange={(e) => update(id, { language: e.target.value })}
                  />
                </label>
              )}
              <label>
                超时（秒）
                <input
                  type="number"
                  min="5"
                  max="300"
                  value={p.timeoutSeconds}
                  onChange={(e) =>
                    update(id, { timeoutSeconds: Number(e.target.value) })
                  }
                />
              </label>
              {(id === "deepseek" || id === "evaluation") && (
                <>
                  <label>
                    Temperature
                    <input
                      type="number"
                      min="0"
                      max="2"
                      step="0.1"
                      value={p.temperature}
                      onChange={(e) =>
                        update(id, { temperature: Number(e.target.value) })
                      }
                    />
                  </label>
                  <label>
                    最大输出 Tokens
                    <input
                      type="number"
                      min="1"
                      max="131072"
                      value={p.maxTokens}
                      onChange={(e) =>
                        update(id, { maxTokens: Number(e.target.value) })
                      }
                    />
                  </label>
                </>
              )}
            </div>
            <div className="credential-row">
              <label>
                {label} API 密钥
                <input
                  type="password"
                  aria-label={`${label} API 密钥`}
                  autoComplete="new-password"
                  value={secrets[id] ?? ""}
                  onChange={(e) =>
                    setSecrets((s) => ({ ...s, [id]: e.target.value }))
                  }
                  placeholder={
                    status[id]
                      ? "已保存，填写新值可替换"
                      : "仅在此处输入，不要发到聊天中"
                  }
                />
              </label>
              <button
                disabled={busy || !secrets[id]?.trim()}
                onClick={() => secret(id)}
              >
                保存密钥
              </button>
              <button
                disabled={busy || !status[id]}
                onClick={() => secret(id, true)}
              >
                清除密钥
              </button>
            </div>
            <button disabled={busy||!p.enabled||!status[id]} onClick={()=>testConnection(id)}>保存并测试 {label} 连接</button>
            {tests[id]&&<p role="status">{tests[id]}</p>}
          </details>
        );
      })}
      <section className="settings-card">
        <h2>音频设备与音量</h2>
        <button onClick={refreshDevices}>授权并刷新音频设备</button>
        <div className="config-grid">
          {(["inputDeviceId", "outputDeviceId"] as const).map((key) => (
            <label key={key}>
              {key === "inputDeviceId" ? "麦克风" : "播放设备"}
              <select
                value={config.audio[key]}
                onChange={(e) =>
                  setConfig({
                    ...config,
                    audio: { ...config.audio, [key]: e.target.value },
                  })
                }
              >
                <option value="">系统默认</option>
                {config.audio[key] &&
                  !devices.some((d) => d.deviceId === config.audio[key]) && (
                    <option value={config.audio[key]}>
                      已保存的设备（刷新可查看名称）
                    </option>
                  )}
                {devices
                  .filter(
                    (d) =>
                      d.kind ===
                      (key === "inputDeviceId" ? "audioinput" : "audiooutput"),
                  )
                  .map((d) => (
                    <option key={d.deviceId} value={d.deviceId}>
                      {d.label || d.deviceId}
                    </option>
                  ))}
              </select>
            </label>
          ))}
          {(["masterVolume", "ttsVolume"] as const).map((key) => (
            <label key={key}>
              {key === "masterVolume" ? "听力 / 主音量" : "考官语音音量"} ·{" "}
              {Math.round(config.audio[key] * 100)}%
              <input
                type="range"
                min="0"
                max="1"
                step=".05"
                value={config.audio[key]}
                onChange={(e) =>
                  setConfig({
                    ...config,
                    audio: { ...config.audio, [key]: Number(e.target.value) },
                  })
                }
              />
            </label>
          ))}
          <label>
            语音采样率
            <select
              value={config.audio.sampleRate}
              onChange={(e) =>
                setConfig({
                  ...config,
                  audio: {
                    ...config.audio,
                    sampleRate: Number(e.target.value),
                  },
                })
              }
            >
              {[16000, 24000, 48000].map((n) => (
                <option key={n} value={n}>
                  {n} Hz
                </option>
              ))}
            </select>
          </label>
          <label>
            TTS 输出格式
            <select
              value={config.audio.format}
              onChange={(e) =>
                setConfig({
                  ...config,
                  audio: { ...config.audio, format: e.target.value },
                })
              }
            >
              {["wav", "mp3", "pcm"].map((s) => (
                <option key={s}>{s}</option>
              ))}
            </select>
          </label>
        </div>
        <p className="muted">
          录音与 TTS
          接入后使用所选设备；当前听力播放器使用主音量。采样率和格式需与选定云模型能力匹配。
        </p>
      </section>
      <section className="settings-card">
        <h2>考场噪音偏好</h2>
        <p className="muted">
          考试或训练开始后按所选强度随机播放本地合成的写字、键盘和饮水设备声；听力期间自动降低环境音量。安静模式关闭环境声。
        </p>
        <div className="config-grid">
          <label>
            强度
            <select
              value={config.noise.profile}
              onChange={(e) =>
                setConfig({
                  ...config,
                  noise: { ...config.noise, profile: e.target.value },
                })
              }
            >
              <option value="clean">安静</option>
              <option value="light">轻度考场</option>
              <option value="challenge">挑战</option>
            </select>
          </label>
          <label>
            噪音音量
            <input
              type="range"
              min="0"
              max="1"
              step=".05"
              value={config.noise.volume}
              onChange={(e) =>
                setConfig({
                  ...config,
                  noise: { ...config.noise, volume: Number(e.target.value) },
                })
              }
            />
          </label>
          {(
            ["handwriting", "keyboard", "waterPump", "speakingEnabled"] as const
          ).map((key, i) => (
            <label key={key} className="check-row">
              <input
                type="checkbox"
                checked={config.noise[key]}
                onChange={(e) =>
                  setConfig({
                    ...config,
                    noise: { ...config.noise, [key]: e.target.checked },
                  })
                }
              />
              {
                [
                  "写字声",
                  "键盘声",
                  "水泵 / 饮水设备声",
                  "口语时也允许环境噪音",
                ][i]
              }
            </label>
          ))}
        </div>
      </section>
      <section className="settings-card">
        <h2>三维场景资产</h2>
        <p className="muted">
          使用随应用提供的考官房间，也可选择独立 GLB。Blender
          源工程用于后续编辑，练习时无需运行 Blender。
        </p>
        {(
          ["executablePath", "sourceBlendPath", "runtimeAssetPath"] as const
        ).map((key, i) => (
          <label className="path-row" key={key}>
            <span>
              {
                [
                  "Blender 程序路径",
                  "Blender 源工程",
                  "运行时 GLB（留空使用内置场景）",
                ][i]
              }
            </span>
            <div>
              <input
                aria-label={
                  ["Blender 程序路径", "Blender 源工程", "运行时 GLB"][i]
                }
                value={config.blender[key]}
                onChange={(e) =>
                  setConfig({
                    ...config,
                    blender: { ...config.blender, [key]: e.target.value },
                  })
                }
              />
              <button
                onClick={async () => {
                  try {
                    const path = await port.chooseFile(
                      [["exe"], ["blend"], ["glb"]][i],
                    );
                    if (path)
                      setConfig({
                        ...config,
                        blender: { ...config.blender, [key]: path },
                      });
                  } catch (e) {
                    setError(errorText(e));
                  }
                }}
              >
                浏览文件…
              </button>
            </div>
          </label>
        ))}
        <label>
          渲染比例 · {config.blender.renderScale.toFixed(1)}
          <input
            type="range"
            min=".5"
            max="2"
            step=".1"
            value={config.blender.renderScale}
            onChange={(e) =>
              setConfig({
                ...config,
                blender: {
                  ...config.blender,
                  renderScale: Number(e.target.value),
                },
              })
            }
          />
        </label>
      </section>
      <div className="config-save">
        <button className="primary" disabled={busy} onClick={save}>
          {busy ? "正在保存…" : "保存全部配置"}
        </button>
        {message && <p role="status">{message}</p>}
        {error && (
          <p className="error" role="alert">
            {error}
          </p>
        )}
      </div>
    </section>
  );
}
