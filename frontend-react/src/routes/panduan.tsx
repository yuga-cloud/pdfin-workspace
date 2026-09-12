import { createFileRoute } from "@tanstack/react-router";
import { Panduan } from "@/features/guide/guide";

export const Route = createFileRoute("/panduan")({
  component: Panduan,
  head: () => ({ meta: [{ title: "Panduan · pdfin" }] }),
});
