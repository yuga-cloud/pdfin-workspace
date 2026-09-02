import type { ReactNode } from "react";
import { Link } from "@tanstack/react-router";
import { ShieldCheck } from "lucide-react";

import { Logo } from "@/components/logo";
import { TOOLS } from "@/lib/tools-catalog";

interface AppShellProps {
  children: ReactNode;
}

const navLinkClass =
  "rounded-xl px-3.5 py-2.5 text-sm font-medium text-muted transition-[background-color,color,transform,box-shadow] duration-200 hover:-translate-y-px hover:bg-surface-2 hover:text-fg";

const navLinkActiveClass =
  "rounded-xl bg-fg px-3.5 py-2.5 text-sm font-semibold text-bg shadow-sm";

const toolLinkClass =
  "shrink-0 rounded-full border border-transparent px-3 py-2 text-xs font-medium text-muted transition-[background-color,border-color,color,transform] duration-200 hover:-translate-y-px hover:border-border hover:bg-surface hover:text-fg";

const toolLinkActiveClass =
  "shrink-0 rounded-full border border-fg bg-fg px-3 py-2 text-xs font-semibold text-bg shadow-sm";

export function AppShell({ children }: AppShellProps) {
  return (
    <div className="min-h-dvh bg-bg text-fg">
      <a
        href="#isi"
        className="sr-only focus:not-sr-only focus:fixed focus:left-4 focus:top-4 focus:z-50 focus:rounded-xl focus:bg-fg focus:px-4 focus:py-2.5 focus:text-sm focus:font-semibold focus:text-bg focus:shadow-lg"
      >
        Loncat ke isi
      </a>

      <header className="sticky top-0 z-40 border-b border-border/70 bg-surface/82 shadow-[0_1px_0_rgba(32,33,36,0.02)] backdrop-blur-xl">
        <div className="mx-auto flex min-h-18 max-w-6xl items-center justify-between gap-4 px-4 sm:px-6">
          <Logo />

          <nav className="hidden items-center gap-1 rounded-2xl bg-bg/70 p-1 md:flex" aria-label="Navigasi utama">
            <Link
              to="/"
              className={`${navLinkClass} font-semibold`}
              activeProps={{ className: navLinkActiveClass }}
              activeOptions={{ exact: true }}
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
            activeProps={{ className: navLinkActiveClass }}
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

      <div className="border-b border-border/65 bg-surface/62">
        <p className="mx-auto flex max-w-6xl items-center gap-2 px-4 py-2 text-xs font-medium text-muted sm:px-6 sm:text-sm">
          <span className="grid size-6 shrink-0 place-items-center rounded-full bg-primary/10">
            <ShieldCheck className="size-3.5 text-primary" aria-hidden="true" />
          </span>
          File tidak pernah diunggah. Semua diproses di HP atau laptop kamu.
        </p>
      </div>

      <main id="isi" className="mx-auto w-full max-w-6xl px-4 py-8 sm:px-6 sm:py-10">
        {children}
      </main>

      <footer className="border-t border-border/80 bg-surface/40">
        <div className="mx-auto flex max-w-6xl flex-col gap-3 px-4 py-8 text-sm text-muted sm:flex-row sm:items-center sm:justify-between sm:px-6">
          <p>
            <span className="font-semibold text-fg">
              pdf<span className="text-primary">in</span>
            </span>{" "}
            &ndash; alat PDF Indonesia, tanpa server menyimpan file.
          </p>

          <p className="rounded-full bg-bg px-3 py-1.5 text-xs font-medium">
            Gelombang 1 &middot; proses di perangkat
          </p>
        </div>
      </footer>
    </div>
  );
}
