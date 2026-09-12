import { createFileRoute } from "@tanstack/react-router";
import { ToolGroupPage } from "@/features/tools/tool-group-page";

export const Route = createFileRoute("/alat/konversi")({
  component: () => <ToolGroupPage group="konversi" />,
  head: () => ({ meta: [{ title: "Konversi · pdfin" }] }),
});
