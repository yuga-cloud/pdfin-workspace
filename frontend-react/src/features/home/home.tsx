import { createFileRoute, Link } from "@tanstack/react-router";
import type { ReactNode } from "react";
import {
  ArrowRight,
  CheckCircle2,
  FileCheck2,
  FileImage,
  FileSpreadsheet,
  FileText,
  Files,
  FileType,
  ImagePlus,
  Layers2,
  LockKeyhole,
  Minimize2,
  MousePointer2,
  Presentation,
  RotateCw,
  Scissors,
  ShieldCheck,
  Stamp,
  Upload,
} from "lucide-react";
import { TOOLS, type ToolDef } from "@/features/tools/catalog";

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
              Alat PDF untuk sehari-hari
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
            Gabung, pisah, kompres, dan ubah dokumen dengan alur yang sederhana. Pilih alat yang sesuai,
            masukkan file, lalu lanjutkan ke hasilnya.
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
            <span><CheckCircle2 className="size-3.5" aria-hidden="true" /> Alur kerja tetap sederhana</span>
          </div>
        </div>

        <ProductPreview />
      </section>

      <section className="home-tools" aria-labelledby="tools-heading">
        <div className="tools-heading">
          <div>
            <p className="section-kicker">Katalog alat</p>
            <h2 id="tools-heading">Pilih yang kamu butuhkan</h2>
            <p className="section-lede">Satu tempat untuk pekerjaan PDF harian, dari merapikan halaman sampai mengubah format.</p>
          </div>
        </div>

        <div className="tool-grid">
          {TOOLS.map((tool) => (
            <ToolCard key={tool.slug} tool={tool} />
          ))}
        </div>
      </section>

      <section className="home-trust" aria-label="Informasi penggunaan">
        <TrustItem icon={<CheckCircle2 aria-hidden="true" />} title="Alur sederhana" body="Pilih alat, masukkan file, dan lanjutkan dalam beberapa langkah." />
        <TrustItem icon={<ShieldCheck aria-hidden="true" />} title="Gunakan seperlunya" body="Masukkan hanya dokumen yang memang perlu kamu olah." />
        <TrustItem icon={<Upload aria-hidden="true" />} title="Tanpa akun" body="Pilih alat, masukkan file, proses, lalu ambil hasilnya." />
      </section>
    </div>
  );
}

function ProductPreview() {
  return (
    <div className="product-preview" aria-hidden="true">
      <div className="product-preview-cursor"><MousePointer2 /></div>
      <div className="product-window">
        <div className="product-window-bar">
          <div className="product-window-dots"><span /><span /><span /></div>
          <span className="product-window-label">pdfin · workspace</span>
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
            </div>

            <div className="product-demo-stage">
              <div className="product-window-drop">
                <span className="product-window-drop-icon"><Upload className="size-5" /></span>
                <div className="product-window-drop-copy">
                  <strong>Letakkan file di sini</strong>
                  <span>atau pilih dari perangkat</span>
                </div>
              </div>

              <div className="product-demo-files">
                <div className="product-demo-file product-demo-file-one">
                  <span className="product-demo-file-icon"><FileText /></span>
                  <span><strong>laporan.pdf</strong><small>4.8 MB</small></span>
                  <CheckCircle2 className="product-demo-file-check" />
                </div>
                <div className="product-demo-file product-demo-file-two">
                  <span className="product-demo-file-icon"><FileCheck2 /></span>
                  <span><strong>invoice.pdf</strong><small>1.9 MB</small></span>
                  <CheckCircle2 className="product-demo-file-check" />
                </div>
              </div>

              <div className="product-demo-progress">
                <div className="product-demo-progress-copy">
                  <span><LoaderDot /> Menggabungkan dokumen</span>
                  <strong>78%</strong>
                </div>
                <div className="product-demo-progress-track"><span /></div>
              </div>

              <div className="product-demo-success">
                <div className="product-demo-success-icon"><CheckCircle2 /></div>
                <div><strong>PDF selesai</strong><span>2 dokumen berhasil digabung</span></div>
                <span className="product-demo-download">Unduh <ArrowRight /></span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function LoaderDot() {
  return <span className="product-demo-loader-dot" aria-hidden="true" />;
}

function ToolCard({ tool }: { tool: ToolDef }) {
  const Icon = ICONS[tool.slug];

  return (
    <Link to="/alat/$slug" params={{ slug: tool.slug }} className="tool-card group">
      <div className="tool-card-top">
        <span className="tool-icon"><Icon className="size-5" aria-hidden="true" /></span>
        <span className="tool-card-arrow"><ArrowRight className="size-4" aria-hidden="true" /></span>
      </div>
      <div className="tool-card-title-row">
        <span className="tool-card-title">{tool.title}</span>
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
