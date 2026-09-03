import type { ReactNode } from "react";
import { Link, useRouterState } from "@tanstack/react-router";
import {
  ArrowLeftRight,
  BookOpen,
  CloudCog,
  Files,
  Layers2,
  Minimize2,
  MonitorCheck,
} from "lucide-react";

import { Logo } from "@/components/logo";
import { getTool } from "@/lib/tools-catalog";

interface AppShellProps {
  children: ReactNode;
}

const navLinkClass =
  "app-nav-link inline-flex shrink-0 items-center gap-2 rounded-xl px-3.5 py-2.5 text-sm font-medium text-muted transition-[background-color,color,transform] duration-200 hover:-translate-y-px hover:bg-surface hover:text-fg";

const navLinkActiveClass =
  "app-nav-link app-nav-link-active inline-flex shrink-0 items-center gap-2 rounded-xl bg-fg px-3.5 py-2.5 text-sm font-semibold text-white shadow-sm";

function NavIcon({ children }: { children: ReactNode }) {
  return (
    <span className="app-nav-icon" aria-hidden="true">
      {children}
    </span>
  );
}

export function AppShell({ children }: AppShellProps) {
  const pathname = useRouterState({ select: (state) => state.location.pathname });
  const currentTool = pathname.startsWith("/alat/")
    ? getTool(pathname.slice("/alat/".length))
    : undefined;
  const activeGroup = currentTool?.group;
  const homeActive = pathname === "/";
  const guideActive = pathname === "/panduan";
  const organizeActive = activeGroup === "atur";
  const optimizeActive = activeGroup === "optimalkan";
  const convertActive = activeGroup === "konversi";

  return (
    <div className="app-shell min-h-dvh bg-bg text-fg">
      <a
        href="#isi"
        className="sr-only focus:not-sr-only focus:fixed focus:left-4 focus:top-4 focus:z-50 focus:rounded-xl focus:bg-fg focus:px-4 focus:py-2.5 focus:text-sm focus:font-semibold focus:text-white focus:shadow-lg"
      >
        Loncat ke isi
      </a>

      <header className="app-header sticky top-0 z-40 border-b border-border/70 bg-surface/90 shadow-[0_1px_0_rgba(32,33,36,0.02)] backdrop-blur-xl">
        <div className="mx-auto flex min-h-18 max-w-6xl flex-wrap items-center gap-3 px-4 py-2 sm:px-6 md:flex-nowrap md:gap-4 md:py-0">
          <Logo className="shrink-0" />

          <nav
            className="app-primary-nav order-2 flex min-w-0 w-full items-center overflow-x-auto md:order-none md:ml-auto md:w-auto"
            aria-label="Navigasi utama"
          >
            <Link
              to="/"
              className={homeActive ? navLinkActiveClass : navLinkClass}
              aria-current={homeActive ? "page" : undefined}
              activeOptions={{ exact: true }}
            >
              <NavIcon><Files /></NavIcon>
              Semua
            </Link>
            <Link
              to="/alat/$slug"
              params={{ slug: "gabung" }}
              className={organizeActive ? navLinkActiveClass : navLinkClass}
              aria-current={organizeActive ? "page" : undefined}
            >
              <NavIcon><Layers2 /></NavIcon>
              Atur PDF
            </Link>
            <Link
              to="/alat/$slug"
              params={{ slug: "kompres" }}
              className={optimizeActive ? navLinkActiveClass : navLinkClass}
              aria-current={optimizeActive ? "page" : undefined}
            >
              <NavIcon><Minimize2 /></NavIcon>
              Optimalkan
            </Link>
            <Link
              to="/alat/$slug"
              params={{ slug: "pdf-ke-word" }}
              className={convertActive ? navLinkActiveClass : navLinkClass}
              aria-current={convertActive ? "page" : undefined}
            >
              <NavIcon><ArrowLeftRight /></NavIcon>
              Konversi
            </Link>
            <Link
              to="/panduan"
              className={guideActive ? navLinkActiveClass : navLinkClass}
              aria-current={guideActive ? "page" : undefined}
              activeOptions={{ exact: true }}
            >
              <NavIcon><BookOpen /></NavIcon>
              Panduan
            </Link>
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
