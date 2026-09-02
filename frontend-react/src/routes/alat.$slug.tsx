import { createFileRoute, Link, notFound } from "@tanstack/react-router";
import { ToolWorkspace } from "@/components/tool-workspace";
import { getTool } from "@/lib/tools-catalog";

export const Route = createFileRoute("/alat/$slug")({
  component: ToolPage,
  loader: ({ params }) => {
    const tool = getTool(params.slug);
    if (!tool) throw notFound();
    return { tool };
  },
  head: ({ loaderData }) => ({
    meta: [
      {
        title: loaderData?.tool
          ? `${loaderData.tool.title} · pdfin`
          : "Alat · pdfin",
      },
    ],
  }),
  notFoundComponent: ToolMissing,
});

function ToolPage() {
  const { tool } = Route.useLoaderData();
  return <ToolWorkspace tool={tool} />;
}

function ToolMissing() {
  return (
    <div className="tool-missing mx-auto max-w-lg rounded-3xl border border-border bg-surface p-8 text-center shadow-[var(--shadow-card)] sm:p-10">
      <span className="mx-auto grid size-12 place-items-center rounded-2xl bg-primary/10 text-primary">
        <span className="text-lg font-bold">!</span>
      </span>
      <p className="mt-5 text-xs font-bold uppercase tracking-[0.16em] text-primary">404</p>
      <h1 className="mt-2 text-2xl font-semibold tracking-tight">Alat tidak ada</h1>
      <p className="mt-2 text-sm leading-relaxed text-muted">
        Mungkin tautannya salah atau alat itu belum masuk gelombang 1.
      </p>
      <Link
        to="/"
        className="mt-6 inline-flex min-h-11 items-center justify-center rounded-xl bg-fg px-4 text-sm font-semibold text-bg shadow-sm transition-transform duration-200 hover:-translate-y-px hover:shadow-md"
      >
        Kembali ke beranda
      </Link>
    </div>
  );
}
