import { useEffect, useState } from "react";
import type { PaperVersion } from "../domain/paper";
import type { DesktopPort } from "../platform/desktop";
import { errorText } from "../platform/desktop";
export function useAsset(
  port: DesktopPort,
  paper: PaperVersion,
  assetId?: string,
) {
  const [url, setUrl] = useState("");
  const [error, setError] = useState("");
  useEffect(() => {
    let cancel = false,
      objectUrl = "";
    setUrl("");
    setError("");
    if (!assetId) return;
    const asset = paper.assets.find((a) => a.id === assetId);
    if (!asset) {
      setError("找不到媒体资源");
      return;
    }
    port
      .readAsset(paper.id, paper.version, assetId)
      .then((bytes) => {
        if (cancel) return;
        objectUrl = URL.createObjectURL(
          new Blob([new Uint8Array(bytes).buffer], { type: asset.mediaType }),
        );
        setUrl(objectUrl);
      })
      .catch((e) => {
        if (!cancel) setError(errorText(e));
      });
    return () => {
      cancel = true;
      if (objectUrl) URL.revokeObjectURL(objectUrl);
    };
  }, [port, paper, assetId]);
  return { url, error };
}
