import { isTauri } from "@tauri-apps/api/core";
let pending:Promise<void>=Promise.resolve();
export async function setGameFullscreen(enabled: boolean): Promise<void> {
  if (!isTauri()) return;
  const operation=pending.catch(()=>{}).then(async()=>{const {getCurrentWindow}=await import("@tauri-apps/api/window");await getCurrentWindow().setFullscreen(enabled);});
  pending=operation;await operation;
}
