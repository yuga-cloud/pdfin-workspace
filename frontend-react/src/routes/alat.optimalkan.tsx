import { createFileRoute } from "@tanstack/react-router";
import { ToolGroupPage } from "@/components/tool-group-page";

export const Route = createFileRoute("/alat/optimalkan")({
  component: () => <ToolGroupPage group="optimalkan" />,
  head: () => ({ meta: [{ title: "Optimalkan · pdfin" }] }),
});
