import { loadPdfjs } from "./engine";

export type PdfThumbnail = {
  url: string;
  width: number;
  height: number;
};

const THUMBNAIL_WIDTH = 180;
const JPEG_QUALITY = 0.68;
const MAX_CACHED_THUMBNAILS = 48;

function createAbortError(): Error {
  const error = new Error("Preview dibatalkan.");
  error.name = "AbortError";
  return error;
}

export class PdfThumbnailCache {
  private readonly file: File;
  private pdfPromise: ReturnType<PdfThumbnailCache["loadDocument"]> | null = null;
  private readonly cache = new Map<number, PdfThumbnail>();
  private disposed = false;

  constructor(file: File) {
    this.file = file;
  }

  private async loadDocument() {
    const pdfjs = await loadPdfjs();
    const data = new Uint8Array(await this.file.arrayBuffer());
    if (this.disposed) throw new Error("Preview sudah dibuang.");
    return pdfjs.getDocument({ data, disableAutoFetch: true }).promise;
  }

  private async getDocument() {
    if (!this.pdfPromise) this.pdfPromise = this.loadDocument();
    return this.pdfPromise;
  }

  async getPageCount(): Promise<number> {
    const pdf = await this.getDocument();
    if (this.disposed) throw new Error("Preview sudah dibuang.");
    return pdf.numPages;
  }

  async render(pageNumber: number, signal?: AbortSignal): Promise<PdfThumbnail> {
    if (pageNumber < 1) throw new RangeError("Nomor halaman harus dimulai dari 1.");
    const cached = this.cache.get(pageNumber);
    if (cached) {
      this.cache.delete(pageNumber);
      this.cache.set(pageNumber, cached);
      return cached;
    }
    if (signal?.aborted) throw createAbortError();

    const pdf = await this.getDocument();
    if (signal?.aborted) throw createAbortError();

    const page = await pdf.getPage(pageNumber);
    let renderTask: ReturnType<typeof page.render> | null = null;
    let abortListener: (() => void) | null = null;

    try {
      const base = page.getViewport({ scale: 1 });
      const scale = THUMBNAIL_WIDTH / Math.max(1, base.width);
      const viewport = page.getViewport({ scale });
      const canvas = document.createElement("canvas");
      const width = Math.max(1, Math.floor(viewport.width));
      const height = Math.max(1, Math.floor(viewport.height));
      canvas.width = width;
      canvas.height = height;
      const context = canvas.getContext("2d");
      if (!context) throw new Error("Browser tidak mendukung canvas.");
      context.fillStyle = "#ffffff";
      context.fillRect(0, 0, width, height);

      renderTask = page.render({ canvas, canvasContext: context, viewport });
      abortListener = () => renderTask?.cancel();
      signal?.addEventListener("abort", abortListener, { once: true });

      try {
        await renderTask.promise;
      } catch (error) {
        if (signal?.aborted) throw createAbortError();
        throw error;
      }

      if (signal?.aborted) throw createAbortError();
      const blob = await new Promise<Blob>((resolve, reject) => {
        canvas.toBlob(
          (value) => value ? resolve(value) : reject(new Error("Gagal membuat pratinjau JPG.")),
          "image/jpeg",
          JPEG_QUALITY,
        );
      });
      canvas.width = 0;
      canvas.height = 0;
      const result = { url: URL.createObjectURL(blob), width, height } satisfies PdfThumbnail;
      if (this.disposed) {
        URL.revokeObjectURL(result.url);
        throw new Error("Preview sudah dibuang.");
      }

      this.cache.set(pageNumber, result);
      while (this.cache.size > MAX_CACHED_THUMBNAILS) {
        const oldestPage = this.cache.keys().next().value;
        if (oldestPage === undefined) break;
        const oldest = this.cache.get(oldestPage);
        this.cache.delete(oldestPage);
        if (oldest) URL.revokeObjectURL(oldest.url);
      }

      return result;
    } finally {
      if (signal && abortListener) signal.removeEventListener("abort", abortListener);
      page.cleanup();
    }
  }

  dispose(): void {
    this.disposed = true;
    for (const thumbnail of this.cache.values()) URL.revokeObjectURL(thumbnail.url);
    this.cache.clear();
    const promise = this.pdfPromise;
    this.pdfPromise = null;
    if (promise) void promise.then((pdf) => pdf.cleanup()).catch(() => undefined);
  }
}
