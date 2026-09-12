import { Link } from "@tanstack/react-router";
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
  Stamp,
} from "lucide-react";

import { GROUPS, TOOLS, type ToolDef } from "@/features/tools/catalog";

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

const GROUP_COPY: Record<ToolDef["group"], { eyebrow: string; description: string }> = {
  atur: {
    eyebrow: "Atur PDF",
    description: "Gabungkan, pisahkan, putar, dan susun halaman PDF dalam satu tempat.",
  },
  optimalkan: {
    eyebrow: "Optimalkan",
    description: "Kecilkan ukuran atau siapkan PDF agar lebih praktis dipakai dan dibagikan.",
  },
  konversi: {
    eyebrow: "Konversi",
    description: "Ubah PDF dan dokumen kantor ke format yang kamu perlukan.",
  },
};

export function ToolGroupPage({ group }: { group: ToolDef["group"] }) {
  const tools = TOOLS.filter((tool) => tool.group === group);
  const copy = GROUP_COPY[group];

  return (
    <div className="tool-group-page">
      <section className="tool-group-hero" aria-labelledby="tool-group-title">
        <p className="tool-group-eyebrow">{copy.eyebrow}</p>
        <h1 id="tool-group-title">Semua alat {copy.eyebrow}</h1>
        <p className="tool-group-description">{copy.description}</p>
      </section>

      <section aria-labelledby="tool-group-grid-title">
        <div className="tool-group-section-heading">
          <div>
            <p className="section-kicker">Katalog</p>
            <h2 id="tool-group-grid-title">Pilih alat</h2>
          </div>
          <span className="tool-group-count">{tools.length} alat</span>
        </div>

        <div className="tool-group-grid">
          {tools.map((tool) => (
            <ToolGroupCard key={tool.slug} tool={tool} />
          ))}
        </div>
      </section>

      <nav className="tool-group-switcher" aria-label="Kategori alat lainnya">
        {GROUPS.map((item) => (
          <Link
            key={item.id}
            to={`/alat/${item.id}` as "/alat/atur" | "/alat/optimalkan" | "/alat/konversi"}
            className={item.id === group ? "is-active" : undefined}
            aria-current={item.id === group ? "page" : undefined}
          >
            {item.label}
            <ArrowRight className="size-3.5" aria-hidden="true" />
          </Link>
        ))}
      </nav>
    </div>
  );
}

function ToolGroupCard({ tool }: { tool: ToolDef }) {
  const Icon = ICONS[tool.slug];

  return (
    <Link to="/alat/$slug" params={{ slug: tool.slug }} className="tool-group-card">
      <div className="tool-group-card-top">
        <span className="tool-group-card-icon">
          <Icon className="size-5" aria-hidden="true" />
        </span>
        <ArrowRight className="tool-group-card-arrow size-4" aria-hidden="true" />
      </div>
      <div className="tool-group-card-title-row">
        <h3>{tool.title}</h3>
      </div>
      <p>{tool.description}</p>
      <span className="tool-group-card-hint">{tool.hint}</span>
    </Link>
  );
}
