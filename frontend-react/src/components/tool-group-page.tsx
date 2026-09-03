import { Link } from "@tanstack/react-router";
import {
  ArrowRight,
  CloudCog,
  FileImage,
  FileSpreadsheet,
  FileText,
  Files,
  FileType,
  ImagePlus,
  Layers2,
  Minimize2,
  MonitorCheck,
  Presentation,
  RotateCw,
  Scissors,
  Stamp,
} from "lucide-react";

import { TOOLS, type ToolDef } from "@/lib/tools-catalog";

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

const GROUP_COPY: Record<ToolDef["group"], { label: string; description: string }> = {
  atur: {
    label: "Atur PDF",
    description: "Gabungkan, pisahkan, putar, dan susun halaman PDF dalam satu tempat.",
  },
  optimalkan: {
    label: "Optimalkan",
    description: "Kecilkan ukuran atau siapkan PDF agar lebih praktis dipakai dan dibagikan.",
  },
  konversi: {
    label: "Konversi",
    description: "Ubah PDF dan dokumen kantor ke format yang kamu perlukan.",
  },
};

export function ToolGroupPage({ group }: { group: ToolDef["group"] }) {
  const tools = TOOLS.filter((tool) => tool.group === group);
  const copy = GROUP_COPY[group];

  return (
    <div className="tool-group-page">
      <section className="tool-group-hero" aria-labelledby="tool-group-title">
        <p className="tool-group-eyebrow">{copy.label}</p>
        <h1 id="tool-group-title">{copy.label}</h1>
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
    </div>
  );
}

function ToolGroupCard({ tool }: { tool: ToolDef }) {
  const Icon = ICONS[tool.slug];
  const server = tool.processing === "server";

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
        <span className={server ? "is-server" : "is-device"}>
          {server ? <CloudCog className="size-3" aria-hidden="true" /> : <MonitorCheck className="size-3" aria-hidden="true" />}
          {server ? "Server" : "Perangkat"}
        </span>
      </div>
      <p>{tool.description}</p>
      <span className="tool-group-card-hint">{tool.hint}</span>
    </Link>
  );
}
