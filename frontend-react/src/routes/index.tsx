import { createFileRoute, Link } from "@tanstack/react-router";
import type { ReactNode } from "react";
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
  LockKeyhole,
  Minimize2,
  MonitorCheck,
  Presentation,
  RotateCw,
  Scissors,
  ShieldCheck,
  Stamp,
  Upload,
} from "lucide-react";
import { TOOLS, type ToolDef } from "@/lib/tools-catalog";

export const Route = createFileRoute("/")({
  component: Home,
});

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

function Home() {
  return (
    <div className="home-page">
      <section className="home-hero home-hero-wrap" aria-labelledby="home-title">
        <div className="home-hero-copy">
          <div className="home-hero-badges">
            <span className="home-eyebrow">
              <ShieldCheck className="size-3.5" aria-hidden="true" />
              PDF tools yang transparan
            </span>
            <span className="home-hero-plain-meta">
              <LockKeyhole className="size-3.5" aria-hidden="true" />
              Tanpa akun
            </span>
          </div>
          <h1 id="home-title" className="home-title">
            Kerja dengan PDF,
            <span> tanpa ribet.</span>
          </h1>
          <p className="home-description">
            Gabung, pisah, kompres, dan ubah dokumen dengan alur yang sederhana. Setiap alat
            menjelaskan apakah file diproses di perangkat atau dikirim ke server.
          </p>
          <div className="home-hero-actions">
            <a className="home-hero-action home-hero-action-primary" href="#tools-heading">
              <Upload className="size-4" aria-hidden="true" />
              Mulai dengan alat
            </a>
            <Link className="home-hero-action home-hero-action-secondary" to="/panduan">
              Lihat panduan
              <ArrowRight className="size-4" aria-hidden="true" />
            </Link>
          </div>
          <div className="home-hero-trust">
            <span><LockKeyhole className="size-3.5" aria-hidden="true" /> Tidak ada akun wajib</span>
            <span><MonitorCheck className="size-3.5" aria-hidden="true" /> Mode pemrosesan selalu ditampilkan</span>
          </div>
        </div>

        <ProductPreview />
      </section>

      <section className="home-tools" aria-labelledby="tools-heading">
        <div className="tools-heading">
          <div>
            <p className="section-kicker">Katalog alat</p>
            <h2 id="tools-heading">Pilih yang kamu butuhkan</h2>
            <p className="section-lede">Satu tempat untuk pekerjaan PDF harian, dengan lokasi pemrosesan yang jelas.</p>
          </div>
        </div>

        <div className="tool-grid">
          {TOOLS.map((tool) => (
            <ToolCard key={tool.slug} tool={tool} />
          ))}
        </div>
      </section>

      <section className="home-trust" aria-label="Informasi privasi dan penggunaan">
        <TrustItem icon={<MonitorCheck aria-hidden="true" />} title="Jelas soal pemrosesan" body="Setiap alat menyebutkan lokasi pemrosesannya sebelum kamu memilih file." />
        <TrustItem icon={<ShieldCheck aria-hidden="true" />} title="Privasi dijelaskan apa adanya" body="Jangan menebak-nebak: baca mode pemrosesan yang tampil di alat yang kamu pilih." />
        <TrustItem icon={<Upload aria-hidden="true" />} title="Mulai tanpa akun" body="Pilih alat, masukkan file, proses, lalu ambil hasilnya." />
      </section>
    </div>
  );
}

function ProductPreview() {
  return (
    <div className="product-preview" aria-hidden="true">
      <div className="product-window">
        <div className="product-window-bar">
          <div className="product-window-dots"><span /><span /><span /></div>
          <span className="product-window-label">pdfin · workspace</span>
          <span className="product-window-bar-status"><MonitorCheck className="size-3" /> Ready</span>
        </div>
        <div className="product-window-body">
          <aside className="product-window-sidebar">
            <span className="product-window-side-caption">TOOLS</span>
            <div className="product-window-side-item product-window-side-item-active"><Files /> Gabung</div>
            <div className="product-window-side-item"><Scissors /> Pisah</div>
            <div className="product-window-side-item"><Minimize2 /> Kompres</div>
            <div className="product-window-side-item"><FileType /> Konversi</div>
          </aside>
          <div className="product-window-main">
            <div className="product-window-title-row">
              <div>
                <span className="product-window-kicker">Atur PDF</span>
                <div className="product-window-title">Gabung PDF</div>
              </div>
              <span className="product-window-status"><CloudCog /> Server</span>
            </div>
            <div className="product-window-drop">
              <span className="product-window-drop-icon"><Upload className="size-5" /></span>
              <div className="product-window-drop-copy">
                <strong>Letakkan file di sini</strong>
                <span>atau pilih dari perangkat</span>
              </div>
            </div>
            <div className="product-window-mini-row">
              <div className="product-window-mini-card">
                <span className="product-window-mini-label">FILES</span>
                <strong className="product-window-mini-value">2 dokumen</strong>
              </div>
              <div className="product-window-mini-card">
                <span className="product-window-mini-label">PROCESSING</span>
                <strong className="product-window-mini-value">Di server</strong>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function ToolCard({ tool }: { tool: ToolDef }) {
  const Icon = ICONS[tool.slug];
  const isServer = tool.processing === "server";

  return (
    <Link to="/alat/$slug" params={{ slug: tool.slug }} className="tool-card group">
      <div className="tool-card-top">
        <span className="tool-icon"><Icon className="size-5" aria-hidden="true" /></span>
        <span className="tool-card-arrow"><ArrowRight className="size-4" aria-hidden="true" /></span>
      </div>
      <div className="tool-card-title-row">
        <span className="tool-card-title">{tool.title}</span>
        <span className={`tool-processing-badge ${isServer ? "is-server" : "is-device"}`}>
          {isServer ? <CloudCog className="size-3" aria-hidden="true" /> : <MonitorCheck className="size-3" aria-hidden="true" />}
          {isServer ? "Server" : "Perangkat"}
        </span>
      </div>
      <span className="tool-card-description">{tool.description}</span>
      <span className="tool-card-hint">{tool.hint}</span>
    </Link>
  );
}

function TrustItem({ icon, title, body }: { icon: ReactNode; title: string; body: string }) {
  return (
    <div className="trust-item">
      <span className="trust-item-icon">{icon}</span>
      <div>
        <p>{title}</p>
        <span>{body}</span>
      </div>
    </div>
  );
}
