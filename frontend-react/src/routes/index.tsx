import { createFileRoute, Link } from "@tanstack/react-router";
import { useMemo, useState } from "react";
import {
  ArrowRight,
  FileImage,
  FileSpreadsheet,
  FileText,
  Files,
  FileType,
  ImagePlus,
  Layers2,
  Minimize2,
  Presentation,
  RotateCw,
  Scissors,
  ShieldCheck,
  Stamp,
} from "lucide-react";
import { GROUPS, TOOLS, type ToolDef } from "@/lib/tools-catalog";

export const Route = createFileRoute("/")({ component: Home });

const ICONS: Record<ToolDef["slug"], typeof Files> = {
  gabung: Files,
  pisah: Scissors,
  halaman: Layers2,
  kompres: Minimize2,
  "jpg-ke-pdf": ImagePlus,
  "pdf-ke-jpg": FileImage,
  putar: RotateCw,
  watermark: Stamp,
  "word-ke-pdf": FileType,
  "excel-ke-pdf": FileSpreadsheet,
  "powerpoint-ke-pdf": Presentation,
  "pdf-ke-word": FileText,
  "pdf-ke-excel": FileSpreadsheet,
  "pdf-ke-powerpoint": Presentation,
};

type Filter = "semua" | ToolDef["group"];

function Home() {
  const [filter, setFilter] = useState<Filter>("semua");
  const visibleTools = useMemo(
    () => (filter === "semua" ? TOOLS : TOOLS.filter((tool) => tool.group === filter)),
    [filter],
  );

  return (
    <div className="home-page pb-8">
      <section className="home-hero relative mx-auto max-w-4xl pt-8 text-center sm:pt-14">
        <span className="home-eyebrow inline-flex items-center gap-2 rounded-full border border-primary/20 bg-primary/8 px-3 py-1.5 text-xs font-semibold text-primary">
          <ShieldCheck className="size-3.5" aria-hidden />
          Pemrosesan lokal, tanpa unggah
        </span>
        <h1 className="home-title mt-6 text-4xl font-semibold tracking-tight text-fg sm:text-6xl">
          Semua alat PDF,
          <span className="block text-primary">di satu tempat.</span>
        </h1>
        <p className="home-description mx-auto mt-5 max-w-2xl text-base leading-relaxed text-muted sm:text-lg">
          Gabung, pisah, kompres, dan ubah dokumen PDF dengan cepat. Pilih alatnya, masukkan file,
          lalu unduh hasilnya.
        </p>
      </section>

      <section className="home-tools mx-auto mt-14 max-w-6xl" aria-labelledby="tools-heading">
        <div className="tools-heading flex flex-col gap-4 border-b border-border pb-4 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <p className="text-xs font-semibold uppercase tracking-[0.16em] text-primary">Katalog alat</p>
            <h2 id="tools-heading" className="mt-1 text-2xl font-semibold tracking-tight text-fg">
              Pilih yang kamu butuhkan
            </h2>
          </div>
          <div className="tool-filter flex gap-1 overflow-x-auto pb-1" role="tablist" aria-label="Kategori alat">
            <FilterTab active={filter === "semua"} onClick={() => setFilter("semua")}>
              Semua
            </FilterTab>
            {GROUPS.map((group) => (
              <FilterTab
                key={group.id}
                active={filter === group.id}
                onClick={() => setFilter(group.id)}
              >
                {group.label}
              </FilterTab>
            ))}
          </div>
        </div>

        <div className="tool-grid mt-6 grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {visibleTools.map((tool) => (
            <ToolCard key={tool.slug} tool={tool} />
          ))}
        </div>
      </section>

      <section className="trust-strip mx-auto mt-16 grid max-w-6xl gap-4 border-t border-border pt-8 sm:grid-cols-3">
        <TrustItem title="Privat secara default" body="File diproses di browser dan tidak disimpan di server." />
        <TrustItem title="Bekerja di HP" body="Tampilan dan tombol dibuat untuk layar kecil maupun laptop." />
        <TrustItem title="Hasil langsung diunduh" body="Tidak perlu akun. Pilih alat, proses, lalu ambil hasilnya." />
      </section>
    </div>
  );
}

function FilterTab({
  active,
  onClick,
  children,
}: {
  active: boolean;
  onClick: () => void;
  children: string;
}) {
  return (
    <button
      type="button"
      role="tab"
      aria-selected={active}
      onClick={onClick}
      className={`shrink-0 rounded-lg px-3 py-2 text-sm font-medium transition-colors ${
        active ? "bg-fg text-bg shadow-sm" : "text-muted hover:bg-surface-2 hover:text-fg"
      }`}
    >
      {children}
    </button>
  );
}

function ToolCard({ tool }: { tool: ToolDef }) {
  const Icon = ICONS[tool.slug];
  return (
    <Link
      to="/alat/$slug"
      params={{ slug: tool.slug }}
      className="tool-card group flex min-h-40 flex-col rounded-2xl border border-border bg-surface p-5 transition-[border-color,box-shadow,transform] duration-150 hover:-translate-y-0.5 hover:border-primary/40 hover:shadow-[var(--shadow-card-hover)]"
    >
      <div className="flex items-start justify-between gap-4">
        <span className="tool-icon grid size-11 place-items-center rounded-xl bg-primary/10 text-primary">
          <Icon className="size-5" aria-hidden />
        </span>
        <ArrowRight className="mt-1 size-4 text-muted transition-transform duration-150 group-hover:translate-x-1 group-hover:text-primary" aria-hidden />
      </div>
      <span className="mt-5 block font-semibold text-fg">{tool.title}</span>
      <span className="mt-1 block text-sm leading-relaxed text-muted">{tool.description}</span>
    </Link>
  );
}

function TrustItem({ title, body }: { title: string; body: string }) {
  return (
    <div className="trust-item border-l-2 border-primary/30 pl-4">
      <p className="font-semibold text-fg">{title}</p>
      <p className="mt-1 text-sm leading-relaxed text-muted">{body}</p>
    </div>
  );
}
