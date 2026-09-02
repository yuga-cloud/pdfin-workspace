import type { ReactNode } from "react";
import { Link } from "@tanstack/react-router";
import {
  ArrowLeftRight,
  BookOpen,
  CloudCog,
  FileImage,
  FileSpreadsheet,
  FileText,
  FileType,
  Files,
  ImagePlus,
  Layers2,
  Minimize2,
  MonitorCheck,
  Presentation,
  RotateCw,
  Scissors,
  Stamp,
} from "lucide-react";

import { Logo } from "@/components/logo";
import { TOOLS, type ToolDef } from "@/lib/tools-catalog";

interface AppShellProps {
  children: ReactNode;
}

const navLinkClass =
  "app-nav-link inline-flex items-center gap-2 rounded-xl px-3.5 py-2.5 text-sm font-medium text-muted transition-[background-color,color,transform,box-shadow] duration-200 hover:-translate-y-px hover:bg-surface-2 hover:text-fg";

const navLinkActiveClass =
  "app-nav-link app-nav-link-active inline-flex items-center gap-2 rounded-xl bg-fg px-3.5 py-2.5 text-sm font-semibold text-white shadow-sm";

const toolLinkClass =
  "app-tool-link inline-flex shrink-0 items-center gap-1.5 rounded-full border border-transparent px-3 py-2 text-xs font-medium text-muted transition-[background-color,border-color,color,transform] duration-200 hover:-translate-y-px hover:border-border hover:bg-surface hover:text-fg";

const toolLinkActiveClass =
  "app-tool-link app-tool-link-active inline-flex shrink-0 items-center gap-1.5 rounded-full border border-fg bg-fg px-3 py-2 text-xs font-semibold text-white shadow-sm";

const TOOL_ICONS: Record<ToolDef["slug"], typeof Files> = {
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

function NavIcon({ children }: { children: ReactNode }) {
  return (
    <span className="app-nav-icon" aria-hidden="true">
      {children}
    </span>
  );
}

export function AppShell({ children }: AppShellProps) {
  return (
    <div className="app-shell min-h-dvh bg-bg text-fg">
      <a
        href="#isi"
        className="sr-only focus:not-sr-only focus:fixed focus:left-4 focus:top-4 focus:z-50 focus:rounded-xl focus:bg-fg focus:px-4 focus:py-2.5 focus:text-sm focus:font-semibold focus:text-white focus:shadow-lg"
      >
        Loncat ke isi
      </a>

      <header className="app-header sticky top-0 z-40 border-b border-border/70 bg-surface/90 shadow-[0_1px_0_rgba(32,33,36,0.02)] backdrop-blur-xl">
        <div className="mx-auto flex min-h-18 max-w-6xl items-center justify-between gap-4 px-4 sm:px-6">
          <Logo />

          <nav className="app-primary-nav hidden items-center gap-1 rounded-2xl border border-border/70 bg-bg/85 p-1 shadow-sm md:flex" aria-label="Navigasi utama">
            <Link
              to="/"
              className={navLinkClass}
              activeProps={{ className: navLinkActiveClass }}
              activeOptions={{ exact: true }}
            >
              <NavIcon><Files /></NavIcon>
              Semua alat
            </Link>
            <Link
              to="/alat/$slug"
              params={{ slug: "gabung" }}
              className={navLinkClass}
              activeProps={{ className: navLinkActiveClass }}
              activeOptions={{ exact: true }}
            >
              <NavIcon><Layers2 /></NavIcon>
              Atur PDF
            </Link>
            <Link
              to="/alat/$slug"
              params={{ slug: "pdf-ke-word" }}
              className={navLinkClass}
              activeProps={{ className: navLinkActiveClass }}
              activeOptions={{ exact: true }}
            >
              <NavIcon><ArrowLeftRight /></NavIcon>
              Konversi
            </Link>
            <Link
              to="/panduan"
              className={navLinkClass}
              activeProps={{ className: navLinkActiveClass }}
              activeOptions={{ exact: true }}
            >
              <NavIcon><BookOpen /></NavIcon>
              Panduan
            </Link>
          </nav>

          <Link
            to="/panduan"
            className={`${navLinkClass} md:hidden`}
            activeProps={{ className: navLinkActiveClass }}
            activeOptions={{ exact: true }}
          >
            <NavIcon><BookOpen /></NavIcon>
            Panduan
          </Link>
        </div>

        <div className="border-t border-border/60 md:hidden">
          <nav className="app-tool-rail mx-auto flex max-w-6xl gap-1 overflow-x-auto px-3 py-2 scrollbar-none" aria-label="Alat">
            {TOOLS.map((tool) => {
              const Icon = TOOL_ICONS[tool.slug];
              return (
                <Link
                  key={tool.slug}
                  to="/alat/$slug"
                  params={{ slug: tool.slug }}
                  className={toolLinkClass}
                  activeProps={{ className: toolLinkActiveClass }}
                  activeOptions={{ exact: true }}
                >
                  <Icon className="app-tool-link-icon" aria-hidden="true" />
                  {tool.short}
                </Link>
              );
            })}
          </nav>
        </div>
      </header>

      <div className="processing-banner border-b border-border/65 bg-surface/72">
        <div className="mx-auto flex max-w-6xl flex-wrap items-center gap-x-4 gap-y-2 px-4 py-2.5 text-xs font-medium text-muted sm:px-6 sm:text-sm">
          <span className="inline-flex items-center gap-2">
            <span className="grid size-6 shrink-0 place-items-center rounded-full bg-primary/10 text-primary">
              <MonitorCheck className="size-3.5" aria-hidden="true" />
            </span>
            Pemrosesan berbeda menurut alat.
          </span>
          <span className="hidden text-border sm:inline" aria-hidden="true">·</span>
          <span className="inline-flex items-center gap-1.5">
            <MonitorCheck className="size-3.5 text-ok" aria-hidden="true" />
            Sebagian diproses di perangkat
          </span>
          <span className="inline-flex items-center gap-1.5">
            <CloudCog className="size-3.5 text-muted" aria-hidden="true" />
            Sebagian diproses di server
          </span>
        </div>
      </div>

      <main id="isi" className="mx-auto w-full max-w-6xl px-4 py-8 sm:px-6 sm:py-10">
        {children}
      </main>

      <footer className="app-footer border-t border-border/80 bg-surface/45">
        <div className="mx-auto flex max-w-6xl flex-col gap-3 px-4 py-8 text-sm text-muted sm:flex-row sm:items-center sm:justify-between sm:px-6">
          <p>
            <span className="font-semibold text-fg">pdf<span className="text-primary">in</span></span>{" "}
            – alat PDF Indonesia dengan pemrosesan di perangkat atau server sesuai alat.
          </p>
          <p className="rounded-full bg-bg px-3 py-1.5 text-xs font-medium">Lokasi pemrosesan ditampilkan di setiap alat</p>
        </div>
      </footer>
    </div>
  );
}
