import {
  useEffect,
  useRef,
  useState,
  type Dispatch,
  type SetStateAction,
} from "react";
import {
  ArrowDown,
  ArrowLeft,
  ArrowUp,
  Download,
  FileText,
  ImageIcon,
  LoaderCircle,
  Trash2,
  Upload,
  X,
} from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { acceptFor, type ToolDef } from "@/lib/tools-catalog";
import {
  countPages,
  processTool,
  renderThumbs,
  revokeObjectUrls,
  type ToolOptions,
} from "@/lib/pdf/engine";
import { cn, formatBytes, uid } from "@/lib/utils";

type Item = {
  id: string;
  file: File;
};

type ResultData = {
  url: string;
  filename: string;
  size: number;
  mime: string;
};

const defaultOptions = (): ToolOptions => ({
  rangeText: "",
  splitEach: false,
  quality: "medium",
  rotateDegrees: 90,
  watermarkText: "DRAFT",
  watermarkOpacity: 0.18,
  addPageNumbers: true,
  pageOrder: null,
});

function getDropLabel(tool: ToolDef): string {
  switch (tool.accept) {
    case "image":
      return "Letakkan gambar di sini, atau pilih dari perangkat";
    case "word":
      return "Letakkan file Word (.docx) di sini, atau pilih dari perangkat";
    case "excel":
      return "Letakkan file Excel (.xlsx) di sini, atau pilih dari perangkat";
    case "powerpoint":
      return "Letakkan file PowerPoint (.pptx) di sini, atau pilih dari perangkat";
    case "pdf":
      return tool.multiple
        ? "Letakkan PDF di sini, atau pilih dari perangkat"
        : "Letakkan satu PDF di sini, atau pilih dari perangkat";
    default:
      return "Letakkan file di sini, atau pilih dari perangkat";
  }
}

function getFileErrorMessage(accept: ToolDef["accept"]): string {
  switch (accept) {
    case "pdf":
      return "Pilih file PDF.";
    case "image":
      return "Pilih gambar JPG, PNG, atau WebP.";
    case "word":
      return "Pilih file Word (.docx).";
    case "excel":
      return "Pilih file Excel (.xlsx).";
    case "powerpoint":
      return "Pilih file PowerPoint (.pptx).";
    default:
      return "Pilih file yang sesuai.";
  }
}

const GROUP_LABELS: Record<ToolDef["group"], string> = {
  atur: "Atur PDF",
  optimalkan: "Optimalkan",
  konversi: "Konversi",
};

export function ToolWorkspace({ tool }: { tool: ToolDef }) {
  return <ToolWorkspaceInner key={tool.slug} tool={tool} />;
}

function ToolWorkspaceInner({ tool }: { tool: ToolDef }) {
  const inputRef = useRef<HTMLInputElement>(null);
  const resultUrlRef = useRef<string | null>(null);
  const thumbUrlsRef = useRef<string[]>([]);
  const [items, setItems] = useState<Item[]>([]);
  const [dragOver, setDragOver] = useState(false);
  const [options, setOptions] = useState<ToolOptions>(defaultOptions());
  const [thumbs, setThumbs] = useState<string[]>([]);
  const [thumbBusy, setThumbBusy] = useState(false);
  const [pageCount, setPageCount] = useState<number | null>(null);
  const [busy, setBusy] = useState(false);
  const [progress, setProgress] = useState({ done: 0, total: 1, label: "" });
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<ResultData | null>(null);

  const accept = acceptFor(tool.accept);
  const inputSize = items.reduce((sum, item) => sum + item.file.size, 0);
  const processingNote =
    tool.processing === "server"
      ? "File dikirim ke server untuk diproses."
      : "File diproses di perangkat ini.";
  const processButtonLabel =
    tool.processing === "server" ? "Proses di server" : "Proses di perangkat ini";

  useEffect(() => {
    return () => {
      const url = resultUrlRef.current;
      if (url) {
        URL.revokeObjectURL(url);
        resultUrlRef.current = null;
      }
      revokeObjectUrls(thumbUrlsRef.current);
      thumbUrlsRef.current = [];
    };
  }, []);

  useEffect(() => {
    const controller = new AbortController();
    let cancelled = false;

    setThumbs((current) => {
      revokeObjectUrls(current);
      thumbUrlsRef.current = [];
      return [];
    });

    async function loadMeta() {
      if (tool.accept !== "pdf" || items.length !== 1) {
        setPageCount(null);
        return;
      }

      const file = items[0]?.file;
      if (!file) {
        setPageCount(null);
        return;
      }

      try {
        const pageTotal = await countPages(file);
        if (!cancelled) setPageCount(pageTotal);
      } catch {
        if (!cancelled) setPageCount(null);
      }

      if (tool.slug !== "halaman") return;

      setThumbBusy(true);
      try {
        const urls = await renderThumbs(file, undefined, controller.signal);
        if (cancelled || controller.signal.aborted) {
          revokeObjectUrls(urls);
          return;
        }

        setThumbs((current) => {
          revokeObjectUrls(current);
          thumbUrlsRef.current = urls;
          return urls;
        });
        setOptions((current) => ({
          ...current,
          pageOrder: urls.map((_, index) => index),
        }));
      } catch (err) {
        if (
          !cancelled &&
          !controller.signal.aborted &&
          !(err instanceof Error && err.name === "AbortError")
        ) {
          setError(err instanceof Error ? err.message : "Gagal membuat pratinjau.");
        }
      } finally {
        if (!cancelled) setThumbBusy(false);
      }
    }

    void loadMeta();
    return () => {
      cancelled = true;
      controller.abort();
    };
  }, [items, tool.accept, tool.slug]);

  const order = options.pageOrder ?? thumbs.map((_, index) => index);

  function replaceResult(next: ResultData | null) {
    const previousUrl = resultUrlRef.current;
    if (previousUrl && previousUrl !== next?.url) URL.revokeObjectURL(previousUrl);
    resultUrlRef.current = next?.url ?? null;
    setResult(next);
  }

  function addFiles(list: FileList | File[]) {
    const next = Array.from(list).filter((file) => {
      switch (tool.accept) {
        case "pdf":
          return file.type === "application/pdf" || /\.pdf$/i.test(file.name);
        case "image":
          return (
            file.type === "image/jpeg" ||
            file.type === "image/png" ||
            file.type === "image/webp" ||
            /\.(png|jpe?g|webp)$/i.test(file.name)
          );
        case "word":
          return (
            file.type.includes("wordprocessingml") ||
            file.type === "application/msword" ||
            /\.docx$/i.test(file.name)
          );
        case "excel":
          return (
            file.type.includes("spreadsheetml") ||
            file.type === "application/vnd.ms-excel" ||
            /\.xlsx?$/i.test(file.name)
          );
        case "powerpoint":
          return file.type.includes("presentationml") || /\.pptx$/i.test(file.name);
        default:
          return false;
      }
    });

    if (next.length === 0) {
      toast.error(getFileErrorMessage(tool.accept));
      return;
    }

    setError(null);
    replaceResult(null);

    if (tool.multiple) {
      setItems((current) => [
        ...current,
        ...next.map((file) => ({ id: uid(), file })),
      ]);
      return;
    }

    setItems([{ id: uid(), file: next[0] }]);
  }

  function move(id: string, direction: -1 | 1) {
    setItems((current) => {
      const index = current.findIndex((item) => item.id === id);
      const target = index + direction;
      if (index < 0 || target < 0 || target >= current.length) return current;
      const copy = [...current];
      [copy[index], copy[target]] = [copy[target], copy[index]];
      return copy;
    });
  }

  function movePage(index: number, direction: -1 | 1) {
    setOptions((current) => {
      const nextOrder = [...(current.pageOrder ?? order)];
      const target = index + direction;
      if (target < 0 || target >= nextOrder.length) return current;
      [nextOrder[index], nextOrder[target]] = [nextOrder[target], nextOrder[index]];
      return { ...current, pageOrder: nextOrder };
    });
  }

  function removePage(index: number) {
    setOptions((current) => {
      const nextOrder = [...(current.pageOrder ?? order)];
      nextOrder.splice(index, 1);
      return { ...current, pageOrder: nextOrder };
    });
  }

  async function run(): Promise<void> {
    if (items.length < tool.minFiles) {
      setError(tool.minFiles > 1 ? `Minimal ${tool.minFiles} file.` : "Pilih file dulu.");
      return;
    }

    setBusy(true);
    setError(null);
    setProgress({ done: 0, total: 1, label: "Menyiapkan…" });

    try {
      const output = await processTool(
        tool.slug,
        items.map((item) => item.file),
        options,
        (done, total, label) => setProgress({ done, total, label }),
      );

      replaceResult({
        url: URL.createObjectURL(output.blob),
        filename: output.filename,
        size: output.blob.size,
        mime: output.mime,
      });
      toast.success("Selesai. File siap diunduh.");
    } catch (err) {
      const message = err instanceof Error ? err.message : "Gagal memproses.";
      setError(message);
      toast.error(message);
    } finally {
      setBusy(false);
    }
  }

  const ready = items.length >= tool.minFiles && !busy;
  const pct = progress.total > 0 ? (progress.done / progress.total) * 100 : 0;
  const dropLabel = getDropLabel(tool);
  const backLabel = `Kembali ke ${GROUP_LABELS[tool.group]}`;

  function handleBack() {
    const sameOriginReferrer =
      typeof document !== "undefined" &&
      typeof window !== "undefined" &&
      document.referrer.startsWith(window.location.origin);

    if (sameOriginReferrer && window.history.length > 1) {
      window.history.back();
      return;
    }

    window.location.assign("/");
  }

  return (
    <div
      className={cn(
        "tool-workspace-page mx-auto max-w-3xl",
        `tool-processing-${tool.processing}`,
        `tool-accept-${tool.accept}`,
      )}
    >
      <button
        type="button"
        onClick={handleBack}
        className="workspace-back-link mb-7 inline-flex items-center gap-2 rounded-lg px-1 py-1 text-sm font-medium text-muted transition-colors hover:text-fg"
      >
        <ArrowLeft className="size-4" aria-hidden />
        {backLabel}
      </button>

      <div className="workspace-tool-heading">
        <div className="workspace-tool-heading-top">
          <p className="workspace-tool-group text-xs font-bold uppercase tracking-[0.14em] text-primary">
            {tool.group === "atur" ? "Atur" : tool.group === "optimalkan" ? "Optimalkan" : "Konversi"}
          </p>
          <span className="workspace-processing-pill">
            {tool.processing === "server" ? "Diproses di server" : "Diproses di perangkat"}
          </span>
        </div>

        <h1 className="workspace-tool-title mt-1 text-3xl font-semibold tracking-tight text-fg sm:text-4xl">
          {tool.title}
        </h1>
        <p className="workspace-tool-description mt-2 max-w-2xl text-muted">{tool.description}</p>
        <p className="workspace-tool-hint mt-1 text-sm text-muted">{tool.hint}</p>
      </div>

      <div
        onDragOver={(event) => {
          event.preventDefault();
          setDragOver(true);
        }}
        onDragLeave={() => setDragOver(false)}
        onDrop={(event) => {
          event.preventDefault();
          setDragOver(false);
          if (event.dataTransfer.files?.length) addFiles(event.dataTransfer.files);
        }}
        className={cn("workspace-dropzone-shell mt-8 rounded-2xl bg-surface p-3", dragOver && "is-dragover")}
      >
        <button
          type="button"
          onClick={() => inputRef.current?.click()}
          className={cn(
            "workspace-dropzone flex w-full flex-col items-center justify-center gap-3 rounded-xl border border-dashed border-border px-4 py-12 text-center",
            dragOver && "is-dragover",
          )}
        >
          <span className="workspace-upload-icon flex size-12 items-center justify-center rounded-2xl bg-surface-2 text-primary">
            {tool.accept === "image" ? <ImageIcon className="size-5" /> : <Upload className="size-5" />}
          </span>
          <span className="text-sm font-semibold text-fg">{dropLabel}</span>
          <span className="workspace-drop-meta text-xs text-muted">
            {processingNote}
          </span>
        </button>

        <input
          ref={inputRef}
          type="file"
          className="sr-only"
          accept={accept}
          multiple={tool.multiple}
          onChange={(event) => {
            if (event.target.files?.length) addFiles(event.target.files);
            event.target.value = "";
          }}
        />

        {items.length > 0 ? (
          <ul className="workspace-file-list mt-3 space-y-2">
            {items.map((item, index) => (
              <li key={item.id} className="workspace-file-row flex items-center gap-3 rounded-lg bg-bg px-3 py-2">
                <span className="workspace-file-icon grid size-9 shrink-0 place-items-center rounded-xl bg-surface text-primary">
                  <FileText className="size-4" aria-hidden />
                </span>
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm font-semibold">{item.file.name}</p>
                  <p className="text-xs text-muted tabular-nums">
                    {formatBytes(item.file.size)}
                    {pageCount && items.length === 1 ? ` · ${pageCount} halaman` : ""}
                  </p>
                </div>
                {tool.multiple ? (
                  <div className="workspace-inline-actions flex items-center gap-1">
                    <Button type="button" variant="ghost" size="icon" className="size-9 min-h-9" aria-label="Naikkan" onClick={() => move(item.id, -1)} disabled={index === 0}>
                      <ArrowUp />
                    </Button>
                    <Button type="button" variant="ghost" size="icon" className="size-9 min-h-9" aria-label="Turunkan" onClick={() => move(item.id, 1)} disabled={index === items.length - 1}>
                      <ArrowDown />
                    </Button>
                  </div>
                ) : null}
                <Button type="button" variant="ghost" size="icon" className="workspace-remove-button size-9 min-h-9" aria-label={`Hapus ${item.file.name}`} onClick={() => setItems((current) => current.filter((entry) => entry.id !== item.id))}>
                  <X />
                </Button>
              </li>
            ))}
          </ul>
        ) : null}
      </div>

      {tool.slug === "halaman" && items.length === 1 ? (
        <section className="workspace-pages mt-6 rounded-2xl bg-surface p-4 sm:p-5">
          <div className="workspace-panel-heading">
            <div>
              <h2 className="text-sm font-semibold">Susun halaman</h2>
              <p className="mt-1 text-xs text-muted">Naik/turun untuk urutan. Hapus yang tidak perlu.</p>
            </div>
            <span className="workspace-panel-count tabular-nums">{order.length} halaman</span>
          </div>

          {thumbBusy ? (
            <p className="mt-4 flex items-center gap-2 text-sm text-muted">
              <LoaderCircle className="size-4 animate-spin" />
              Membuat pratinjau…
            </p>
          ) : (
            <ol className="workspace-page-grid mt-4 grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4">
              {order.map((pageIndex, index) => (
                <li key={`${pageIndex}-${index}`} className="workspace-page-card rounded-lg bg-bg p-2">
                  {thumbs[pageIndex] ? (
                    <img src={thumbs[pageIndex]} alt={`Halaman ${pageIndex + 1}`} className="aspect-3/4 w-full rounded-md object-contain outline -outline-offset-1 outline-fg/10" />
                  ) : (
                    <div className="flex aspect-3/4 items-center justify-center rounded-md bg-surface-2 text-xs text-muted">{pageIndex + 1}</div>
                  )}
                  <div className="mt-2 flex items-center justify-between gap-1">
                    <span className="text-xs tabular-nums text-muted">Halaman {pageIndex + 1}</span>
                    <div className="flex">
                      <button type="button" className="grid size-8 place-items-center rounded-md hover:bg-surface-2" aria-label="Naikkan halaman" onClick={() => movePage(index, -1)} disabled={index === 0}><ArrowUp className="size-3.5" /></button>
                      <button type="button" className="grid size-8 place-items-center rounded-md hover:bg-surface-2" aria-label="Turunkan halaman" onClick={() => movePage(index, 1)} disabled={index === order.length - 1}><ArrowDown className="size-3.5" /></button>
                      <button type="button" className="grid size-8 place-items-center rounded-md text-danger hover:bg-surface-2" aria-label="Hapus halaman" onClick={() => removePage(index)}><Trash2 className="size-3.5" /></button>
                    </div>
                  </div>
                </li>
              ))}
            </ol>
          )}
        </section>
      ) : null}

      <OptionsPanel tool={tool} options={options} setOptions={setOptions} pageCount={pageCount} />

      {error ? <p className="workspace-error mt-4 rounded-lg px-3 py-2 text-sm" role="alert">{error}</p> : null}

      {busy ? (
        <div className="workspace-progress mt-6">
          <div className="mb-2 flex items-center justify-between text-xs text-muted">
            <span>{progress.label}</span>
            <span className="tabular-nums">{Math.round(pct)}%</span>
          </div>
          <Progress value={pct} />
        </div>
      ) : null}

      <div className="workspace-process-row mt-6 flex flex-col gap-3 sm:flex-row sm:items-center">
        <Button type="button" size="lg" onClick={() => void run()} disabled={!ready} className="workspace-process-button w-full sm:w-auto">
          {busy ? <><LoaderCircle className="animate-spin" /> Memproses</> : processButtonLabel}
        </Button>
        {items.length > 0 ? <p className="text-xs text-muted tabular-nums">{items.length} file · {formatBytes(inputSize)}</p> : null}
      </div>

      {result ? (
        <div className="workspace-result mt-8 rounded-2xl px-5 py-5 text-bg">
          <div className="workspace-result-heading flex items-start justify-between gap-4">
            <div>
              <p className="text-sm font-semibold">Siap diunduh</p>
              <p className="mt-1 text-sm text-bg/70">{result.filename} · <span className="tabular-nums">{formatBytes(result.size)}</span>{inputSize > 0 && result.mime === "application/pdf" ? <> · dari {formatBytes(inputSize)}</> : null}</p>
            </div>
            <span className="workspace-result-check">✓</span>
          </div>
          <div className="mt-4 flex flex-col gap-2 sm:flex-row">
            <Button type="button" className="workspace-download-button bg-primary-fg text-fg hover:opacity-90" onClick={() => { const anchor = document.createElement("a"); anchor.href = result.url; anchor.download = result.filename; anchor.click(); }}><Download /> Unduh hasil</Button>
            <Button type="button" variant="ghost" className="text-bg hover:bg-bg/10 hover:text-bg" onClick={() => { setItems([]); replaceResult(null); setOptions(defaultOptions()); }}>Proses file lain</Button>
          </div>
        </div>
      ) : null}
    </div>
  );
}

function OptionsPanel({ tool, options, setOptions, pageCount }: { tool: ToolDef; options: ToolOptions; setOptions: Dispatch<SetStateAction<ToolOptions>>; pageCount: number | null }) {
  if (tool.slug === "gabung" || tool.slug === "jpg-ke-pdf" || tool.slug === "halaman") return null;

  return (
    <section className="workspace-options mt-6 rounded-2xl bg-surface p-4 sm:p-5">
      <div className="workspace-panel-heading">
        <div>
          <h2 className="text-sm font-semibold">Pengaturan</h2>
          <p className="mt-1 text-xs text-muted">Sesuaikan hasil sebelum diproses.</p>
        </div>
      </div>

      {tool.slug === "pisah" ? (
        <div className="mt-4 space-y-4">
          <label className="workspace-check-row flex items-center gap-3 text-sm">
            <input type="checkbox" className="size-4 accent-primary" checked={options.splitEach} onChange={(event) => setOptions((current) => ({ ...current, splitEach: event.target.checked }))} />
            Pecah tiap halaman jadi file PDF terpisah (ZIP)
          </label>
          {options.splitEach ? null : (
            <label className="block text-sm">
              <span className="text-muted">Rentang halaman {pageCount ? `(1–${pageCount})` : ""}</span>
              <input value={options.rangeText} onChange={(event) => setOptions((current) => ({ ...current, rangeText: event.target.value }))} placeholder="1-3, 5, 8-10" className="workspace-field mt-1 h-11 w-full rounded-md border border-border bg-bg px-3 text-sm outline-none" />
            </label>
          )}
        </div>
      ) : null}

      {tool.slug === "kompres" ? (
        <fieldset className="mt-4">
          <legend className="text-sm text-muted">Kualitas</legend>
          <div className="workspace-option-grid mt-3 grid gap-2 sm:grid-cols-3">
            {([
              ["high", "Halus", "Lebih tajam, file lebih besar"],
              ["medium", "Sedang", "Seimbang, disarankan"],
              ["low", "Kecil", "Paling hemat, untuk WhatsApp"],
            ] as const).map(([value, label, hint]) => (
              <label key={value} className={cn("workspace-choice cursor-pointer rounded-lg border px-3 py-3 text-sm", options.quality === value && "is-selected")}>
                <input type="radio" name="quality" className="sr-only" checked={options.quality === value} onChange={() => setOptions((current) => ({ ...current, quality: value }))} />
                <span className="font-semibold">{label}</span>
                <span className="mt-1 block text-xs text-muted">{hint}</span>
              </label>
            ))}
          </div>
        </fieldset>
      ) : null}

      {tool.slug === "putar" ? (
        <fieldset className="mt-4">
          <legend className="text-sm text-muted">Putar semua halaman</legend>
          <div className="workspace-segmented mt-3 flex flex-wrap gap-2">
            {([90, 180, 270] as const).map((degrees) => (
              <button key={degrees} type="button" onClick={() => setOptions((current) => ({ ...current, rotateDegrees: degrees }))} className={cn("h-11 rounded-md px-4 text-sm", options.rotateDegrees === degrees && "is-selected")}>{degrees}°</button>
            ))}
          </div>
        </fieldset>
      ) : null}

      {tool.slug === "watermark" ? (
        <div className="mt-4 space-y-4">
          <label className="block text-sm">
            <span className="text-muted">Teks stempel</span>
            <input value={options.watermarkText} onChange={(event) => setOptions((current) => ({ ...current, watermarkText: event.target.value }))} placeholder="DRAFT, ASLI, RAHASIA" className="workspace-field mt-1 h-11 w-full rounded-md border border-border bg-bg px-3 text-sm outline-none" />
          </label>
          <div className="workspace-chip-row flex flex-wrap gap-2">
            {["DRAFT", "ASLI", "RAHASIA", "KOPI"].map((watermark) => (
              <button key={watermark} type="button" className="workspace-chip h-9 rounded-full px-3 text-xs font-medium" onClick={() => setOptions((current) => ({ ...current, watermarkText: watermark }))}>{watermark}</button>
            ))}
          </div>
          <label className="block text-sm">
            <span className="flex items-center justify-between gap-4 text-muted"><span>Ketebalan stempel</span><span className="tabular-nums text-fg">{Math.round(options.watermarkOpacity * 100)}%</span></span>
            <input type="range" min={0.08} max={0.4} step={0.02} value={options.watermarkOpacity} onChange={(event) => setOptions((current) => ({ ...current, watermarkOpacity: Number(event.target.value) }))} className="mt-2 w-full accent-primary" />
          </label>
          <label className="workspace-check-row flex items-center gap-3 text-sm">
            <input type="checkbox" className="size-4 accent-primary" checked={options.addPageNumbers} onChange={(event) => setOptions((current) => ({ ...current, addPageNumbers: event.target.checked }))} />
            Tambahkan nomor halaman
          </label>
        </div>
      ) : null}

      {(tool.slug === "pdf-ke-jpg" || tool.slug === "word-ke-pdf" || tool.slug === "powerpoint-ke-pdf" || tool.slug === "pdf-ke-word" || tool.slug === "pdf-ke-excel" || tool.slug === "pdf-ke-powerpoint" || tool.slug === "excel-ke-pdf") ? (
        <div className="workspace-info-note mt-4 rounded-xl bg-bg px-3 py-3 text-xs leading-relaxed text-muted">
          Mode pemrosesan: {tool.processing === "server" ? "server" : "perangkat"}. Detail dan batasan hasil mengikuti alat yang kamu pilih.
        </div>
      ) : null}
    </section>
  );
}
