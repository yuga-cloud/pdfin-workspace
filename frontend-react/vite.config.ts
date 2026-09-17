import { defineConfig } from "vite";
import { tanstackStart } from "@tanstack/react-start/plugin/vite";
import viteReact from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { nitro } from "nitro/vite";
import { pwaPlugin } from "./scripts/pwa-plugin.ts";
import { appEnvPlugin } from "./scripts/app-env-plugin.ts";

const devPublicHost = process.env.PDFIN_DEV_PUBLIC_HOST?.trim();

export default defineConfig(({ command, isPreview }) => ({
  server: {
    host: "0.0.0.0",
    port: 8080,
    strictPort: true,
    ...(devPublicHost ? { allowedHosts: [devPublicHost] } : {}),
    proxy: {
      "/rust-api": {
        target: "http://127.0.0.1:3000",
        changeOrigin: true,
      },
    },
  },
  preview: {
    host: "127.0.0.1",
    port: 8081,
    strictPort: true,
  },
  resolve: { tsconfigPaths: true },
  plugins: [
    appEnvPlugin(),
    pwaPlugin(),
    tailwindcss(),
    tanstackStart(),
    ...(command === "build" || isPreview
      ? [
          nitro({
            preset: "vercel",
            serverDir: "./server",
          }),
        ]
      : []),
    viteReact(),
  ],
  test: {
    include: ["src/**/*.{test,spec}.{ts,tsx}"],
    exclude: ["scripts/**/*.test.{mjs,ts,tsx}"],
  },
}));
