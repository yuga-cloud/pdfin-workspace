import { createFileRoute } from "@tanstack/react-router";
import { ToolGroupPage } from "@/components/tool-group-page";

export const Route = createFileRoute("/alat/konversi")({
  component: () => <ToolGroupPage group="konversi" />,
  head: () => ({ meta: [{ title: "Konversi · pdfin" }] }),
});
