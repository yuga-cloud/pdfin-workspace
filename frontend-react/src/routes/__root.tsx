import { createRootRoute, HeadContent, Outlet, Scripts } from "@tanstack/react-router";
import { PreviewHostBridge } from "@/shared/preview/preview-host-bridge";
import { AppShell } from "@/app/app-shell";
import { Toaster } from "sonner";
import appCss from "../styles.css?url";

const APP_NAME = "pdfin";

export const Route = createRootRoute({
  head: () => ({
    meta: [
      { charSet: "utf-8" },
      { name: "viewport", content: "width=device-width, initial-scale=1" },
      { title: APP_NAME },
      {
        name: "description",
        content:
          "Alat PDF Indonesia untuk menggabung, memisah, mengompres, memutar, memberi watermark, dan mengonversi dokumen. Lokasi pemrosesan ditampilkan sesuai alat.",
      },
      { name: "theme-color", content: "#f7f7f5" },
      { name: "color-scheme", content: "light" },
    ],
    links: [
      { rel: "icon", type: "image/svg+xml", href: "/favicon.svg" },
      { rel: "stylesheet", href: appCss },
      { rel: "manifest", href: "/__app/manifest.webmanifest" },
      { rel: "apple-touch-icon", href: "/__app/icon-180.png" },
    ],
  }),
  component: RootDocument,
});

function RootDocument() {
  return (
    <html lang="id" className="antialiased" suppressHydrationWarning>
      <head>
        <HeadContent />
      </head>
      <body className="min-h-dvh bg-bg font-sans text-fg">
        <PreviewHostBridge />
        <AppShell>
          <Outlet />
        </AppShell>
        <Toaster
          position="bottom-center"
          toastOptions={{
            className: "font-sans",
          }}
        />
        <Scripts />
      </body>
    </html>
  );
}
