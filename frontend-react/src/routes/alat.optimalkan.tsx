import { createFileRoute } from "@tanstack/react-router";
import { ToolGroupPage } from "@/features/tools/tool-group-page";

export const Route = createFileRoute("/alat/optimalkan")({
  component: () => <ToolGroupPage group="optimalkan" />,
  head: () => ({ meta: [{ title: "Optimalkan · pdfin" }] }),
});
