import type { ErrorComponentProps } from "@tanstack/react-router";
import { ArrowLeft, TriangleAlert } from "lucide-react";

export function AppErrorComponent({ error }: ErrorComponentProps) {
  return (
    <main className="flex min-h-screen flex-col items-center justify-center bg-bg px-6 py-12 text-center text-fg">
      <div className="w-full max-w-lg rounded-3xl border border-border bg-surface p-8 shadow-[var(--shadow-card)] sm:p-10">
        <span className="mx-auto grid size-12 place-items-center rounded-2xl border border-primary/15 bg-primary/10 text-primary" aria-hidden="true">
          <TriangleAlert className="size-6" strokeWidth={2} />
        </span>
        <p className="mt-5 text-xs font-bold uppercase tracking-[0.16em] text-primary">Terjadi kesalahan</p>
        <h1 className="mt-2 text-2xl font-semibold tracking-tight">Halaman tidak dapat dimuat</h1>
        <p className="mt-3 break-words text-sm leading-relaxed text-muted">
          {error.message || "Terjadi kesalahan yang tidak terduga. Coba muat ulang halaman atau kembali ke beranda."}
        </p>
        <a
          href="/"
          className="mt-6 inline-flex min-h-11 items-center justify-center gap-2 rounded-xl bg-fg px-4 text-sm font-semibold text-bg shadow-sm transition-[transform,box-shadow,opacity] duration-200 hover:-translate-y-px hover:shadow-md focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/40 focus-visible:ring-offset-2 focus-visible:ring-offset-bg"
        >
          <ArrowLeft className="size-4" aria-hidden="true" />
          Kembali ke beranda
        </a>
      </div>
    </main>
  );
}
