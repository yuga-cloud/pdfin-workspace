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
    <div className="mx-auto max-w-lg py-12 text-center">
      <h1 className="text-2xl font-semibold">Alat tidak ada</h1>
      <p className="mt-2 text-muted">Mungkin tautannya salah atau alat itu belum masuk gelombang 1.</p>
      <Link to="/" className="mt-6 inline-block text-sm font-medium text-primary hover:underline">
        Kembali ke beranda
      </Link>
    </div>
  );
}
