import type { ReactNode } from "react";
import { Link } from "@tanstack/react-router";
import { ShieldCheck } from "lucide-react";

import { Logo } from "@/components/logo";
import { TOOLS } from "@/lib/tools-catalog";

interface AppShellProps {
  children: ReactNode;
}

const navLinkClass =
  "rounded-md px-3 py-2 text-sm text-muted transition-colors hover:bg-surface-2 hover:text-fg";

const navLinkActiveClass =
  "rounded-md bg-surface-2 px-3 py-2 text-sm text-fg";

const toolLinkClass =
  "shrink-0 rounded-full px-3 py-2 text-xs font-medium text-muted transition-colors hover:bg-surface-2 hover:text-fg";

const toolLinkActiveClass =
  "shrink-0 rounded-full bg-fg px-3 py-2 text-xs font-medium text-bg hover:bg-fg hover:text-bg";

export function AppShell({ children }: AppShellProps) {
  return (
    <div className="min-h-dvh bg-bg text-fg">
      <a
        href="#isi"
        className="sr-only focus:not-sr-only focus:absolute focus:left-4 focus:top-4 focus:z-50 focus:rounded-md focus:bg-fg focus:px-3 focus:py-2 focus:text-bg"
      >
        Loncat ke isi
      </a>

      <header className="border-b border-border/80 bg-surface/90 backdrop-blur-sm">
        <div className="mx-auto flex h-18 max-w-6xl items-center justify-between gap-4 px-4">
          <Logo />

          <nav
            className="hidden items-center gap-1 md:flex"
            aria-label="Navigasi utama"
          >
            <Link
              to="/"
              className={`${navLinkClass} font-medium`}
              activeProps={{ className: `${navLinkActiveClass} font-medium` }}
            >
              Semua alat
            </Link>

            <Link
              to="/alat/$slug"
              params={{ slug: "gabung" }}
              className={navLinkClass}
              activeProps={{ className: navLinkActiveClass }}
            >
              Atur PDF
            </Link>

            <Link
              to="/alat/$slug"
              params={{ slug: "pdf-ke-word" }}
              className={navLinkClass}
              activeProps={{ className: navLinkActiveClass }}
            >
              Konversi
            </Link>

            <Link
              to="/panduan"
              className={navLinkClass}
              activeProps={{ className: navLinkActiveClass }}
            >
              Panduan
            </Link>
          </nav>

          <Link
            to="/panduan"
            className={`${navLinkClass} md:hidden`}
            activeProps={{ className: `${navLinkActiveClass} md:hidden` }}
          >
            Panduan
          </Link>
        </div>

        <div className="border-t border-border/60 md:hidden">
          <nav
            className="mx-auto flex max-w-6xl gap-1 overflow-x-auto px-3 py-2 scrollbar-none"
            aria-label="Alat"
          >
            {TOOLS.map((tool) => (
              <Link
                key={tool.slug}
                to="/alat/$slug"
                params={{ slug: tool.slug }}
                className={toolLinkClass}
                activeProps={{ className: toolLinkActiveClass }}
              >
                {tool.short}
              </Link>
            ))}
          </nav>
        </div>
      </header>

      <div className="border-b border-border/70 bg-surface-2/60">
        <p className="mx-auto flex max-w-6xl items-center gap-2 px-4 py-2 text-xs text-muted sm:text-sm">
          <ShieldCheck
            className="size-4 shrink-0 text-primary"
            aria-hidden="true"
          />
          File tidak pernah diunggah. Semua diproses di HP atau laptop kamu.
        </p>
      </div>

      <main
        id="isi"
        className="mx-auto w-full max-w-6xl px-4 py-8 sm:py-10"
      >
        {children}
      </main>

      <footer className="border-t border-border/80">
        <div className="mx-auto flex max-w-6xl flex-col gap-3 px-4 py-8 text-sm text-muted sm:flex-row sm:items-center sm:justify-between">
          <p>
            <span className="font-medium text-fg">
              pdf<span className="text-primary">in</span>
            </span>{" "}
            &ndash; alat PDF Indonesia, tanpa server menyimpan file.
          </p>

          <p>Gelombang 1 &middot; proses di perangkat</p>
        </div>
      </footer>
    </div>
  );
}
