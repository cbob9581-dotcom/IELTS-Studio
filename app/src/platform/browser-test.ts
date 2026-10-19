// Only imported in Vite's testbrowser mode, never a production fallback.
import type { DesktopPort } from "./desktop";
import type { AttemptSnapshot } from "../domain/session";
import type { PaperVersion } from "../domain/paper";
import reading from "../../test/fixtures/dev-reading.json";
import { defaultConfig, type AppConfig } from "../configuration/types";
const papers = [reading] as unknown as PaperVersion[];
const read = (): AttemptSnapshot[] =>
  JSON.parse(localStorage.getItem("ielts-test-attempts") ?? "[]");
const write = (items: AttemptSnapshot[]) =>
  localStorage.setItem("ielts-test-attempts", JSON.stringify(items));
export const browserTestPort: DesktopPort = {
  cloudCreate:async()=>{throw Error("真实云服务需从桌面应用调用");},cloudList:async()=>[],
  cloudRead:async()=>{throw Error("桌面云任务未连接");},cloudStep:async()=>{throw Error("桌面云任务未连接");},
  cloudCancel:async()=>{},cloudReview:async()=>{throw Error("桌面云任务未连接");},cloudPublish:async()=>{throw Error("桌面云任务未连接");},
  cloudAddMedia:async()=>{throw Error("桌面云任务未连接");},cloudTest:async()=>{throw Error("真实连接测试需桌面应用");},
  cloudAsset:async()=>{throw Error("草稿媒体需桌面应用");},
  synthesizeSpeech:async()=>{throw Error("Cloud speech is not faked in browser storage tests");},
  recognizeSpeech:async()=>{throw Error("Cloud speech requires desktop");},
  examinerQuestion:async()=>{throw Error("Cloud speech requires desktop");},
  saveSpeakingRecording:async()=>{throw Error("Cloud recordings require desktop");},
  cancelSpeech:async()=>{},
  getConfig: async () =>
    JSON.parse(
      localStorage.getItem("ielts-test-config") ??
        JSON.stringify(defaultConfig()),
    ) as AppConfig,
  saveConfig: async (config) => {
    localStorage.setItem("ielts-test-config", JSON.stringify(config));
  },
  credentialStatus: async () => ({}),
  saveCredential: async () => {
    throw Error("Browser test cannot save OS credentials");
  },
  clearCredential: async () => {},
  chooseFile: async () => null,
  readRuntimeModel: async () => {
    throw Error("Native runtime model requires desktop");
  },
  getRoots: async () => ({
    assets: "TEST ONLY assets",
    study: "TEST ONLY study",
  }),
  chooseDirectory: async () => null,
  setRoots: async (x) => x,
  migrateRoots: async (x) => x,
  importPaper: async () => null,
  listPapers: async () =>
    papers.map((p) => ({ ...p, sections: p.sections.map((s) => s.section) })),
  loadPaper: async (id, version) => {
    const p = papers.find((x) => x.id === id && x.version === version);
    if (!p) throw Error("not found");
    return structuredClone(p);
  },
  saveAttempt: async (a, r) => {
    const all = read();
    const old = all.find((x) => x.id === a.id);
    if ((old?.revision ?? 0) !== r) throw Error("revision_conflict");
    const v = { ...structuredClone(a), revision: r + 1 };
    write([...all.filter((x) => x.id !== a.id), v]);
    return v;
  },
  loadAttempt: async (id) => {
    const a = read().find((a) => a.id === id);
    if (!a) throw Error("not found");
    return a;
  },
  listAttempts: async () => read(),
  submitAttempt: async (id, r) => {
    const all = read();
    const old = all.find((a) => a.id === id)!;
    if (old.status === "submitted") return old;
    if (old.revision !== r) throw Error("revision_conflict");
    const v = { ...old, status: "submitted" as const, revision: r + 1 };
    write([...all.filter((x) => x.id !== id), v]);
    return v;
  },
  readAsset: async () => {
    throw Error("test asset unavailable");
  },
};
