import { loadPdfjs } from "./engine";

export type PdfThumbnail = {
  url: string;
  width: number;
  height: number;
};

const THUMBNAIL_WIDTH = 180;
const JPEG_QUALITY = 0.68;
const MAX_CACHED_THUMBNAILS = 48;
const MAX_CONCURRENT_RENDERS = 4;

const MAX_THUMBNAIL_PAGES = 1_000;
const MAX_THUMBNAIL_PIXELS = 20_000_000;
const MAX_THUMBNAIL_INPUT_BYTES = 100 * 1024 * 1024;

function createAbortError(): Error {
  const error = new Error("Preview dibatalkan.");
  error.name = "AbortError";
  return error;
}

export class PdfThumbnailCache {
  private readonly file: File;
  private pdfPromise: ReturnType<PdfThumbnailCache["loadDocument"]> | null = null;
  private readonly cache = new Map<number, PdfThumbnail>();
  private activeRenders = 0;
  private readonly renderWaiters: Array<{
    signal?: AbortSignal;
    resolve: () => void;
    reject: (error: Error) => void;
    onAbort: () => void;
  }> = [];
  private disposed = false;
  private documentCleanupStarted = false;

  constructor(file: File) {
    this.file = file;
  }

  private async loadDocument() {
    const pdfjs = await loadPdfjs();
    if (this.file.size > MAX_THUMBNAIL_INPUT_BYTES) {
      throw new Error(
        "PDF terlalu besar untuk pratinjau di browser (maksimum 100 MiB).",
      );
    }

    const data = new Uint8Array(await this.file.arrayBuffer());
    if (this.disposed) throw new Error("Preview sudah dibuang.");
    return pdfjs.getDocument({
      data,
      disableAutoFetch: true,
      disableStream: true,
      stopAtErrors: true,
      maxImageSize: MAX_THUMBNAIL_PIXELS,
    }).promise;
  }

  private async acquireRenderSlot(signal?: AbortSignal): Promise<void> {
    if (signal?.aborted) throw createAbortError();
    if (this.disposed) throw new Error("Preview sudah dibuang.");

    if (this.activeRenders < MAX_CONCURRENT_RENDERS) {
      this.activeRenders += 1;
      return;
    }

    await new Promise<void>((resolve, reject) => {
      const waiter = {
        signal,
        resolve: () => {
          signal?.removeEventListener("abort", onAbort);
          resolve();
        },
        reject: (error: Error) => {
          signal?.removeEventListener("abort", onAbort);
          reject(error);
        },
        onAbort: () => {
          const index = this.renderWaiters.indexOf(waiter);
          if (index !== -1) this.renderWaiters.splice(index, 1);
          reject(createAbortError());
        },
      };
      const onAbort = () => waiter.onAbort();

      if (signal) {
        signal.addEventListener("abort", onAbort, { once: true });
        if (signal.aborted) {
          waiter.onAbort();
          return;
        }
      }

      this.renderWaiters.push(waiter);
    });

    if (this.disposed) {
      this.releaseRenderSlot();
      throw new Error("Preview sudah dibuang.");
    }
  }

  private releaseRenderSlot(): void {
    this.activeRenders -= 1;

    while (this.renderWaiters.length > 0) {
      const waiter = this.renderWaiters.shift();
      if (!waiter) return;
      if (waiter.signal?.aborted) {
        waiter.reject(createAbortError());
        continue;
      }
      this.activeRenders += 1;
      waiter.resolve();
      return;
    }
  }

  private rejectRenderWaiters(): void {
    while (this.renderWaiters.length > 0) {
      const waiter = this.renderWaiters.shift();
      waiter?.reject(new Error("Preview sudah dibuang."));
    }
  }

  private async getDocument() {
    if (!this.pdfPromise) this.pdfPromise = this.loadDocument();
    return this.pdfPromise;
  }

  private cleanupDocumentWhenIdle(): void {
    if (!this.disposed || this.activeRenders !== 0 || this.documentCleanupStarted) return;
    const promise = this.pdfPromise;
    if (!promise) return;

    this.documentCleanupStarted = true;
    this.pdfPromise = null;
    void promise.then((pdf) => pdf.cleanup()).catch(() => undefined);
  }

  async getPageCount(): Promise<number> {
    const pdf = await this.getDocument();

    try {
      if (this.disposed) {
        throw new Error("Preview sudah dibuang.");
      }

      if (pdf.numPages === 0) {
        throw new Error("PDF tidak memiliki halaman.");
      }

      if (pdf.numPages > MAX_THUMBNAIL_PAGES) {
        throw new Error(
          `PDF memiliki terlalu banyak halaman untuk pratinjau (maksimum ${MAX_THUMBNAIL_PAGES}).`,
        );
      }

      return pdf.numPages;
    } catch (error) {
      this.pdfPromise = null;
      await pdf.cleanup().catch(() => undefined);
      throw error;
    }
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

    await this.acquireRenderSlot(signal);
    try {
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

        if (
          width > 16_384 ||
          height > 16_384 ||
          width * height > MAX_THUMBNAIL_PIXELS
        ) {
          throw new Error("Ukuran halaman PDF terlalu besar untuk pratinjau.");
        }

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
    } finally {
      this.releaseRenderSlot();
      this.cleanupDocumentWhenIdle();
    }
  }

  dispose(): void {
    this.disposed = true;
    this.rejectRenderWaiters();
    for (const thumbnail of this.cache.values()) URL.revokeObjectURL(thumbnail.url);
    this.cache.clear();
    this.cleanupDocumentWhenIdle();
  }
}