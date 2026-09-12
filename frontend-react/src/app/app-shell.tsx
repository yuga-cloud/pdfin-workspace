import type { ReactNode } from "react";
import { Link, useRouterState } from "@tanstack/react-router";
import {
  ArrowLeftRight,
  BookOpen,
  Files,
  Layers2,
  Minimize2,
} from "lucide-react";

import { Logo } from "@/shared/brand/logo";
import { getTool, type ToolDef } from "@/features/tools/catalog";

interface AppShellProps {
  children: ReactNode;
}

const navLinkClass =
  "app-nav-link inline-flex shrink-0 items-center gap-2 rounded-xl px-3.5 py-2.5 text-sm font-medium text-muted transition-[color,transform] duration-200 hover:-translate-y-px hover:text-fg";

const navLinkActiveClass =
  "app-nav-link app-nav-link-active inline-flex shrink-0 items-center gap-2 rounded-xl px-3.5 py-2.5 text-sm font-semibold text-fg";

function NavIcon({ children }: { children: ReactNode }) {
  return (
    <span className="app-nav-icon" aria-hidden="true">
      {children}
    </span>
  );
}

function groupFromPath(pathname: string): ToolDef["group"] | undefined {
  if (pathname === "/alat/atur" || pathname === "/alat/optimalkan" || pathname === "/alat/konversi") {
    return pathname.slice("/alat/".length) as ToolDef["group"];
  }

  if (pathname.startsWith("/alat/")) {
    return getTool(pathname.slice("/alat/".length))?.group;
  }

  return undefined;
}

export function AppShell({ children }: AppShellProps) {
  const pathname = useRouterState({ select: (state) => state.location.pathname });
  const activeGroup = groupFromPath(pathname);
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

      <header className="app-header sticky top-0 z-40 border-b border-border/70 bg-surface/95 shadow-[0_1px_0_rgba(32,33,36,0.02)]">
        <div className="app-header-inner mx-auto flex min-h-18 max-w-6xl items-center px-4 py-2 sm:px-6 md:py-0">
          <Logo className="app-header-logo shrink-0" />

          <div className="app-primary-nav-shell">
            <nav className="app-primary-nav" aria-label="Navigasi utama">
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
                to="/alat/atur"
                className={organizeActive ? navLinkActiveClass : navLinkClass}
                aria-current={organizeActive ? "page" : undefined}
              >
                <NavIcon><Layers2 /></NavIcon>
                Atur PDF
              </Link>
              <Link
                to="/alat/optimalkan"
                className={optimizeActive ? navLinkActiveClass : navLinkClass}
                aria-current={optimizeActive ? "page" : undefined}
              >
                <NavIcon><Minimize2 /></NavIcon>
                Optimalkan
              </Link>
              <Link
                to="/alat/konversi"
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
        </div>
      </header>

      <main id="isi" className="mx-auto w-full max-w-6xl px-4 py-8 sm:px-6 sm:py-10">
        {children}
      </main>

      <footer className="app-footer border-t border-border/80 bg-surface/45">
        <div className="mx-auto flex max-w-6xl flex-col gap-3 px-4 py-8 text-sm text-muted sm:flex-row sm:items-center sm:justify-between sm:px-6">
          <p className="app-footer-tagline">
            <span className="app-footer-brand">pdf<span>in</span></span>
            <span className="app-footer-separator" aria-hidden="true">–</span>
            <span className="app-footer-copy">alat PDF untuk pekerjaan dokumen sehari-hari.</span>
          </p>
        </div>
      </footer>
    </div>
  );
}
