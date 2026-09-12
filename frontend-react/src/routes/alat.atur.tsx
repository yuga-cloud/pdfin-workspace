import { createFileRoute } from "@tanstack/react-router";
import { ToolGroupPage } from "@/features/tools/tool-group-page";

export const Route = createFileRoute("/alat/atur")({
  component: () => <ToolGroupPage group="atur" />,
  head: () => ({ meta: [{ title: "Atur PDF · pdfin" }] }),
});
