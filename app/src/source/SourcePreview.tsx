import { useEffect, useRef, useState } from "react";
import type { DesktopPort } from "../platform/desktop";
import { errorText } from "../platform/desktop";
import type { Block, SourceDocument } from "./types";
export function SourcePreview({
  port,
  projectId,
  document,
  block,
}: {
  port: DesktopPort;
  projectId: string;
  document: SourceDocument;
  block?: Block;
}) {
  const canvas = useRef<HTMLCanvasElement>(null),
    [page, setPage] = useState(block?.page ?? 1),
    [pages, setPages] = useState(0),
    [error, setError] = useState(""),
    [image, setImage] = useState(""),
    [zoom, setZoom] = useState(1);
  const pdfRef = useRef<import("pdfjs-dist").PDFDocumentProxy | null>(null);
  const [loaded, setLoaded] = useState(0);
  useEffect(() => {
    if (block?.page) setPage(block.page);
  }, [block]);
  useEffect(() => {
    setPage(block?.page ?? 1);
    setPages(0);
    setError("");
    let cancelled = false;
    let url = "";
    let task: import("pdfjs-dist").PDFDocumentLoadingTask | undefined;
    (async () => {
      if (!port.sourceAsset) return;
      if (document.extension === "pdf") {
        const pdf = await import("pdfjs-dist");
        pdf.GlobalWorkerOptions.workerSrc = new URL(
          "pdfjs-dist/build/pdf.worker.min.mjs",
          import.meta.url,
        ).toString();
        const bytes = await port.sourceAsset(projectId, document.id, null);
        if (cancelled) return;
        task = pdf.getDocument({
          data: new Uint8Array(bytes),
          enableXfa: false,
          useSystemFonts: true,
        });
        const doc = await task.promise;
        if (cancelled) {
          await task.destroy();
          return;
        }
        pdfRef.current = doc;
        setPages(doc.numPages);
        setLoaded((n) => n + 1);
      } else if (["png", "jpg", "jpeg", "webp"].includes(document.extension)) {
        const bytes = await port.sourceAsset(projectId, document.id, null);
        if (cancelled) return;
        url = URL.createObjectURL(new Blob([bytes]));
        setImage(url);
      }
    })().catch((e) => {
      if (!cancelled) setError(errorText(e));
    });
    return () => {
      cancelled = true;
      pdfRef.current = null;
      void task?.destroy();
      if (url) URL.revokeObjectURL(url);
      setImage("");
    };
  }, [port, projectId, document.id]);
  useEffect(() => {
    const doc = pdfRef.current;
    if (!doc || !canvas.current) return;
    let cancelled = false;
    let render: import("pdfjs-dist").RenderTask | undefined;
    (async () => {
      const p = await doc.getPage(Math.min(Math.max(1, page), doc.numPages));
      if (cancelled || !canvas.current) return;
      const c = canvas.current,
        viewport = p.getViewport({ scale: 1.25 * zoom });
      c.width = viewport.width;
      c.height = viewport.height;
      render = p.render({ canvas: c, viewport });
      await render.promise;
    })().catch((e) => {
      if (!cancelled && e?.name !== "RenderingCancelledException")
        setError(errorText(e));
    });
    return () => {
      cancelled = true;
      render?.cancel();
    };
  }, [loaded, page, zoom]);
  return (
    <section className="source-preview">
      <h3>原文件 · {document.name}</h3>
      {document.extension === "pdf" && (
        <>
          <div className="library-actions">
            <button disabled={page <= 1} onClick={() => setPage((n) => n - 1)}>
              上一页
            </button>
            <label>
              PDF 页码
              <input
                type="number"
                min={1}
                max={pages || 1}
                value={page}
                onChange={(e) =>
                  setPage(
                    Math.max(1, Math.min(pages || 1, Number(e.target.value))),
                  )
                }
              />
            </label>
            <span>/ {pages || "读取中"}</span>
            <button
              disabled={page >= pages}
              onClick={() => setPage((n) => n + 1)}
            >
              下一页
            </button>
            <button onClick={() => setZoom((z) => (z === 1 ? 1.8 : 1))}>
              缩放
            </button>
          </div>
          <div className="source-canvas-wrap">
            <canvas ref={canvas} />
            {block?.page === page && block.bbox && (
              <div
                className="source-highlight"
                style={{
                  left: `${block.bbox[0] * 100}%`,
                  top: `${block.bbox[1] * 100}%`,
                  width: `${(block.bbox[2] - block.bbox[0]) * 100}%`,
                  height: `${(block.bbox[3] - block.bbox[1]) * 100}%`,
                }}
              />
            )}
          </div>
        </>
      )}
      {image && (
        <img className="source-original-image" src={image} alt="原始资料" />
      )}
      {block && (
        <>
          <p>
            来源：{block.page ? `PDF 第 ${block.page} 页` : "文本资料"} ·{" "}
            {block.id}
          </p>
          <pre className="source-text">{block.text}</pre>
        </>
      )}
      {error && (
        <p role="alert" className="error">
          原件预览未完成：{error}
        </p>
      )}
    </section>
  );
}
