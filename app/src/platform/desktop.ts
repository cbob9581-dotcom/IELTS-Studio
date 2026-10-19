import { invoke, isTauri } from "@tauri-apps/api/core";
import { open, confirm } from "@tauri-apps/plugin-dialog";
import type { PaperVersion, Section } from "../domain/paper";
import type { AttemptSnapshot } from "../domain/session";
import type { AppConfig, ProviderId } from "../configuration/types";
import type { CloudJob } from "../cloud/types";
export type StorageRoots = { assets: string; study: string };
export type DesktopError = {
  code: string;
  message: string;
  retryable: boolean;
};
export type PaperSummary = Pick<
  PaperVersion,
  "id" | "version" | "title" | "status" | "provenance"
> & { sections: Section[] };
export interface DesktopPort {
  prepareSourcePages?(id:string,documentId:string,pages:number[]):Promise<void>;
  releaseSourcePages?():void;
  sourceAction?(action: string, payload: Record<string, unknown>): Promise<any>;
  sourceStep?(
    id: string,
    targetId: string,
  ): Promise<import("../source/types").SourceProject>;
  sourceAsset?(
    id: string,
    documentId: string,
    assetId: string | null,
  ): Promise<ArrayBuffer>;
  chooseFiles?(extensions: string[]): Promise<string[]>;
  cloudCreate(
    kind: CloudJob["kind"],
    request: Record<string, unknown>,
    path: string | null,
  ): Promise<CloudJob>;
  cloudList(): Promise<CloudJob[]>;
  cloudProgress?(id: string): Promise<CloudJob>;
  cloudRead(id: string): Promise<CloudJob>;
  cloudStep(id: string): Promise<CloudJob>;
  cloudCancel(id: string): Promise<void>;
  cloudReview(id: string, paper: PaperVersion): Promise<CloudJob>;
  cloudRecheck?(id: string): Promise<CloudJob>;
  cloudPublish(id: string): Promise<CloudJob>;
  cloudAddMedia(id: string, path: string): Promise<CloudJob>;
  cloudTest(
    provider: ProviderId,
  ): Promise<{ detail: string; latencyMs: number; testedAt: number }>;
  cloudAsset(id: string, assetId: string): Promise<ArrayBuffer>;
  synthesizeSpeech(
    attemptId: string,
    turnId: string,
    text: string,
    requestId: string,
  ): Promise<ArrayBuffer>;
  recognizeSpeech(
    attemptId: string,
    takeId: string,
    requestId: string,
  ): Promise<string>;
  examinerQuestion(
    prompt: string,
    previous: string,
    part: number,
    requestId: string,
  ): Promise<string>;
  saveSpeakingRecording(
    attemptId: string,
    takeId: string,
    wavBase64: string,
  ): Promise<void>;
  recoverSpeakingRecording?(
    attemptId: string,
    takeId: string,
    chunks: string[],
  ): Promise<{ takeId: string; voiced: boolean; durationMs: number }>;
  readSpeakingRecording?(
    attemptId: string,
    takeId: string,
  ): Promise<ArrayBuffer>;
  cancelSpeech(requestId: string): Promise<void>;
  getConfig(): Promise<AppConfig>;
  saveConfig(config: AppConfig): Promise<void>;
  credentialStatus(): Promise<Partial<Record<ProviderId, boolean>>>;
  saveCredential(provider: ProviderId, secret: string): Promise<void>;
  clearCredential(provider: ProviderId): Promise<void>;
  chooseFile(extensions: string[]): Promise<string | null>;
  readRuntimeModel(): Promise<ArrayBuffer>;
  getRoots(): Promise<StorageRoots | null>;
  chooseDirectory(): Promise<string | null>;
  setRoots(roots: StorageRoots): Promise<StorageRoots>;
  importPaper(): Promise<PaperVersion | null>;
  listPapers(): Promise<PaperSummary[]>;
  loadPaper(id: string, version: number): Promise<PaperVersion>;
  saveAttempt(
    a: AttemptSnapshot,
    expectedRevision: number,
  ): Promise<AttemptSnapshot>;
  loadAttempt(id: string): Promise<AttemptSnapshot>;
  listAttempts(): Promise<AttemptSnapshot[]>;
  submitAttempt(id: string, expectedRevision: number): Promise<AttemptSnapshot>;
  migrateRoots(target: StorageRoots): Promise<StorageRoots>;
  readAsset(id: string, version: number, assetId: string): Promise<number[]>;
  readAssetBinary?(id:string,version:number,assetId:string):Promise<ArrayBuffer>;
}
const native: DesktopPort = {
  readAssetBinary:(id,version,assetId)=>invoke('read_asset_binary',{id,version,assetId}),
  prepareSourcePages:async(id,documentId,pages)=>{const m=await import("../source/pdfPages");await m.prepareSourcePages(native,id,documentId,pages);},
  releaseSourcePages:()=>{void import("../source/pdfPages").then(m=>m.releaseSourcePages(native));},
  sourceAction: (action, payload) =>
    invoke("source_action", { action, payload }),
  sourceStep: (id, targetId) => invoke("source_step", { id, targetId }),
  sourceAsset: (id, documentId, assetId) =>
    invoke("source_asset", { id, documentId, assetId }),
  chooseFiles: async (extensions) => {
    const paths = await open({
      multiple: true,
      filters: [{ name: "资料文件", extensions }],
    });
    return typeof paths === "string" ? [paths] : (paths ?? []);
  },
  cloudCreate: (kind, request, path) =>
    invoke("cloud_create", { kind, request, path }),
  cloudList: () => invoke("cloud_list"),
  cloudProgress: (id) => invoke("cloud_progress", { id }),
  cloudRead: (id) => invoke("cloud_read", { id }),
  cloudStep: (id) => invoke("cloud_step", { id }),
  cloudCancel: (id) => invoke("cloud_cancel", { id }),
  cloudReview: (id, paper) => invoke("cloud_review", { id, paper }),
  cloudPublish: (id) => invoke("cloud_publish", { id }),
  cloudRecheck: (id) => invoke("cloud_recheck", { id }),
  cloudAddMedia: (id, path) => invoke("cloud_add_media", { id, path }),
  cloudTest: (provider) => invoke("cloud_test", { provider }),
  cloudAsset: (id, assetId) => invoke("cloud_asset", { id, assetId }),
  synthesizeSpeech: (attemptId, turnId, text, requestId) =>
    invoke("speech_synthesize", { attemptId, turnId, text, requestId }),
  recognizeSpeech: (attemptId, takeId, requestId) =>
    invoke("speech_recognize", { attemptId, takeId, requestId }),
  examinerQuestion: (prompt, previous, part, requestId) =>
    invoke("speech_examiner", { prompt, previous, part, requestId }),
  saveSpeakingRecording: (attemptId, takeId, wavBase64) =>
    invoke("save_speaking_recording", { attemptId, takeId, wavBase64 }),
  recoverSpeakingRecording: (attemptId, takeId, chunks) =>
    invoke("recover_speaking_recording", { attemptId, takeId, chunks }),
  readSpeakingRecording: (attemptId, takeId) =>
    invoke("read_speaking_recording", { attemptId, takeId }),
  cancelSpeech: (requestId) => invoke("cancel_speech", { requestId }),
  getConfig: () => invoke("get_config"),
  saveConfig: (config) => invoke("save_config", { config }),
  credentialStatus: () => invoke("credential_status"),
  saveCredential: (provider, secret) =>
    invoke("save_credential", { provider, secret }),
  clearCredential: (provider) => invoke("clear_credential", { provider }),
  chooseFile: async (extensions) => {
    const path = await open({
      multiple: false,
      filters: [{ name: "本地文件", extensions }],
    });
    return typeof path === "string" ? path : null;
  },
  readRuntimeModel: () => invoke("read_runtime_model"),
  getRoots: () => invoke("get_roots"),
  chooseDirectory: async () => {
    const p = await open({
      directory: true,
      multiple: false,
      title: "选择保存目录",
    });
    return typeof p === "string" ? p : null;
  },
  setRoots: (roots) => invoke("set_roots", { roots }),
  importPaper: async () => {
    const path = await open({
      multiple: false,
      filters: [{ name: "IELTS JSON 卷包", extensions: ["json"] }],
      title: "导入试卷 JSON（资源须在相同目录内）",
    });
    return typeof path === "string" ? invoke("import_paper", { path }) : null;
  },
  listPapers: () => invoke("list_papers"),
  loadPaper: (id, version) => invoke("load_paper", { id, version }),
  saveAttempt: (snapshot, expectedRevision) =>
    invoke("save_attempt", { snapshot, expectedRevision }),
  loadAttempt: (id) => invoke("load_attempt", { id }),
  listAttempts: () => invoke("list_attempts"),
  submitAttempt: (id, expectedRevision) =>
    invoke("submit_attempt", { id, expectedRevision }),
  migrateRoots: (target) => invoke("migrate_roots", { target }),
  readAsset: (id, version, assetId) =>
    invoke("read_asset", { id, version, assetId }),
};
export async function getDesktop(): Promise<DesktopPort> {
  if (isTauri()) return native;
  if (import.meta.env.MODE === "testbrowser") {
    const { browserTestPort } = await import("./browser-test");
    return browserTestPort;
  }
  throw new Error("请从桌面程序打开。本地文件与学习记录功能需要 Tauri。");
}
export async function confirmAction(message: string): Promise<boolean> {
  return isTauri()
    ? confirm(message, { title: "IELTS Studio", kind: "warning" })
    : window.confirm(message);
}
export function errorText(e: unknown): string {
  if (e && typeof e === "object" && "message" in e) return String(e.message);
  return String(e);
}
export class SaveQueue {
  private pending: AttemptSnapshot | null = null;
  private running: Promise<void> | null = null;
  private failure: unknown = null;
  constructor(
    private save: (a: AttemptSnapshot, r: number) => Promise<AttemptSnapshot>,
    public revision: number,
  ) {}
  get dirty() {
    return !!this.pending || !!this.running || !!this.failure;
  }
  enqueue(a: AttemptSnapshot) {
    this.pending = structuredClone(a);
    this.failure = null;
  }
  async flush(): Promise<void> {
    if (this.running) {
      await this.running;
      if (this.pending) return this.flush();
      return;
    }
    this.running = (async () => {
      while (this.pending) {
        const draft = this.pending;
        this.pending = null;
        try {
          const saved = await this.save(
            { ...draft, revision: this.revision },
            this.revision,
          );
          this.revision = saved.revision;
          this.failure = null;
        } catch (e) {
          this.pending = this.pending ?? draft;
          this.failure = e;
          throw e;
        }
      }
    })();
    try {
      await this.running;
    } finally {
      this.running = null;
    }
  }
}
